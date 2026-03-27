//! Shadow database — apply schema + migrations to an isolated in-memory SurrealDB
//! for verification without affecting any real database.

use surrealdb::{Surreal, engine::local::Mem};
use tracing::info;

use crate::manifest::Manifest;
use crate::validate::{SchemaDiff, compare_db_info};

/// Result of applying a manifest to a shadow database.
#[derive(Debug)]
pub struct ShadowResult {
	pub db_info: serde_json::Value,
	pub applied_modules: usize,
	pub applied_migrations: usize,
	pub errors: Vec<String>,
}

/// Apply all schema modules and migrations from a manifest to a fresh in-memory SurrealDB.
///
/// Returns the final `INFO FOR DB` state and counts. Collects errors per-file
/// but continues applying to capture as much state as possible.
///
/// The in-memory database is dropped when this function returns.
/// No cleanup needed -- Rust ownership handles deallocation.
pub async fn apply_to_shadow(manifest: &Manifest) -> crate::Result<ShadowResult> {
	let db = Surreal::new::<Mem>(())
		.await
		.map_err(crate::Error::Database)?;
	db.use_ns(&manifest.meta.ns)
		.use_db(&manifest.meta.db)
		.await
		.map_err(crate::Error::Database)?;

	let mut applied_modules = 0;
	let mut applied_migrations = 0;
	let mut errors = Vec::new();

	let modules = match &manifest.preloaded_modules {
		Some(m) => m.clone(),
		None => crate::schema::load_schema_modules(manifest)?,
	};
	for module in &modules {
		match db.query(&module.content).await {
			Ok(r) => match r.check() {
				Ok(_) => applied_modules += 1,
				Err(e) => errors.push(format!("schema/{}: {e}", module.name)),
			},
			Err(e) => errors.push(format!("schema/{}: {e}", module.name)),
		}
	}

	let migrations = if let Some(ref preloaded) = manifest.preloaded_migrations {
		preloaded.clone()
	} else if let Ok(root) = manifest.root_path() {
		crate::migration::discover_migrations(root)?
	} else {
		Vec::new()
	};

	for mig in &migrations {
		match db.query(mig.content.as_str()).await {
			Ok(r) => match r.check() {
				Ok(_) => applied_migrations += 1,
				Err(e) => errors.push(format!("{}: {e}", mig.name)),
			},
			Err(e) => errors.push(format!("{}: {e}", mig.name)),
		}
	}

	let mut response = db
		.query("INFO FOR DB")
		.await
		.map_err(crate::Error::Database)?;
	let db_info: Option<serde_json::Value> = response
		.take(0)
		.map_err(|e| crate::Error::Validation(format!("failed to read shadow DB info: {e}")))?;
	let db_info = db_info.ok_or_else(|| {
		crate::Error::Validation("INFO FOR DB returned no data from shadow".into())
	})?;

	info!(
		modules = applied_modules,
		migrations = applied_migrations,
		errors = errors.len(),
		"Shadow DB apply complete"
	);

	Ok(ShadowResult {
		db_info,
		applied_modules,
		applied_migrations,
		errors,
	})
}

/// Apply manifest to a shadow DB, then compare with a target's INFO FOR DB.
///
/// Returns a `SchemaDiff` showing tables/functions that differ between
/// what the manifest produces and what the target database has.
pub async fn verify_against_shadow(
	manifest: &Manifest,
	target_db_info: &serde_json::Value,
) -> crate::Result<SchemaDiff> {
	let shadow = apply_to_shadow(manifest).await?;
	Ok(compare_db_info(&shadow.db_info, target_db_info))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::Manifest;

	#[tokio::test]
	async fn should_apply_manifest_to_shadow() {
		let manifest = Manifest::builder()
			.meta("test", "shadow_test", "_system")
			.module("core", &[], &["DEFINE TABLE user SCHEMAFULL;\nDEFINE FIELD name ON user TYPE string;\nDEFINE FUNCTION fn::greet($name: string) { RETURN 'Hello, ' + $name; };"])
			.migration(1, "seed", "CREATE user:alice SET name = 'Alice';")
			.build()
			.unwrap();

		let result = apply_to_shadow(&manifest).await.unwrap();

		assert_eq!(result.applied_modules, 1);
		assert_eq!(result.applied_migrations, 1);
		assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

		let tables = result
			.db_info
			.get("tables")
			.and_then(|t| t.as_object())
			.expect("should have tables");
		assert!(tables.contains_key("user"), "should have user table");

		let functions = result
			.db_info
			.get("functions")
			.and_then(|f| f.as_object())
			.expect("should have functions");
		assert!(
			functions.keys().any(|k| k.contains("greet")),
			"should have greet function, got: {:?}",
			functions.keys().collect::<Vec<_>>()
		);
	}

	#[tokio::test]
	async fn should_verify_matching_schemas() {
		let manifest = Manifest::builder()
			.meta("test", "verify_match", "_system")
			.module(
				"core",
				&[],
				&["DEFINE TABLE post SCHEMAFULL;\nDEFINE TABLE comment SCHEMAFULL;"],
			)
			.build()
			.unwrap();

		let shadow = apply_to_shadow(&manifest).await.unwrap();
		let diff = verify_against_shadow(&manifest, &shadow.db_info)
			.await
			.unwrap();

		assert!(diff.is_empty(), "identical schemas should match: {diff}");
	}

	#[tokio::test]
	async fn should_detect_missing_table_in_target() {
		let manifest = Manifest::builder()
			.meta("test", "detect_missing", "_system")
			.module(
				"core",
				&[],
				&["DEFINE TABLE user SCHEMAFULL;\nDEFINE TABLE post SCHEMAFULL;"],
			)
			.build()
			.unwrap();

		let target = serde_json::json!({
			"tables": {
				"user": "DEFINE TABLE user TYPE NORMAL SCHEMAFULL"
			},
			"functions": {}
		});

		let diff = verify_against_shadow(&manifest, &target).await.unwrap();

		assert!(
			diff.missing_tables.contains(&"post".to_string()),
			"should detect post missing from target: {diff}"
		);
	}

	#[tokio::test]
	async fn should_verify_after_apply() {
		let manifest = Manifest::builder()
			.meta("test", "verify_apply", "_system")
			.module(
				"core",
				&[],
				&["DEFINE TABLE user SCHEMAFULL;\nDEFINE FIELD name ON user TYPE string;"],
			)
			.migration(1, "seed", "CREATE user:alice SET name = 'Alice';")
			.build()
			.unwrap();

		let shadow = apply_to_shadow(&manifest).await.unwrap();
		assert!(shadow.errors.is_empty(), "errors: {:?}", shadow.errors);

		let diff = verify_against_shadow(&manifest, &shadow.db_info)
			.await
			.unwrap();
		assert!(
			diff.is_empty(),
			"applying the same manifest twice should produce identical schema: {diff}"
		);
	}

	#[tokio::test]
	async fn should_report_migration_errors_without_failing() {
		let manifest = Manifest::builder()
			.meta("test", "error_handling", "_system")
			.module("core", &[], &["DEFINE TABLE user SCHEMALESS;"])
			.migration(1, "good", "CREATE user:alice SET name = 'Alice';")
			.migration(2, "bad", "THIS IS NOT VALID SQL !!!")
			.build()
			.unwrap();

		let result = apply_to_shadow(&manifest).await.unwrap();

		assert_eq!(result.applied_modules, 1);
		assert!(
			!result.errors.is_empty(),
			"should have errors from bad migration"
		);
		assert!(
			result.errors.iter().any(|e| e.contains("bad")),
			"error should mention 'bad'"
		);
	}
}
