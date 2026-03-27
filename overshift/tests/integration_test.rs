//! Full integration tests — require either Docker or in-memory SurrealDB.

#![cfg(any(feature = "validate-docker", feature = "validate-mem"))]

mod common;

use overshift::Manifest;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

fn fixture_path() -> std::path::PathBuf {
	std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_project")
}

fn fixture() -> Manifest {
	Manifest::load(fixture_path()).unwrap()
}

/// Unique NS per test to avoid interference.
fn test_manifest(test_name: &str) -> Manifest {
	let mut manifest = fixture();
	manifest.meta.ns = format!("test_{test_name}");
	manifest
}

fn connect() -> Surreal<Any> {
	#[cfg(feature = "validate-docker")]
	{
		return common::docker::connect();
	}
	#[cfg(all(feature = "validate-mem", not(feature = "validate-docker")))]
	{
		return common::mem::connect();
	}
}

fn runtime() -> &'static tokio::runtime::Runtime {
	#[cfg(feature = "validate-docker")]
	{
		return common::docker::runtime();
	}
	#[cfg(all(feature = "validate-mem", not(feature = "validate-docker")))]
	{
		return common::mem::runtime();
	}
}

// ─── Core plan + apply flow ───

#[test]
fn full_plan_and_apply() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("full_plan_and_apply");

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		assert_eq!(plan.pending_migrations.len(), 2);
		assert_eq!(plan.schema_modules.len(), 2);
		assert!(!plan.functions_to_validate.is_empty());

		let result = plan.apply(&db).await.unwrap();
		assert_eq!(result.applied_migrations, 2);
		assert_eq!(result.applied_modules, 2);

		// Second plan: nothing pending
		let plan2 = overshift::plan(&db, &manifest).await.unwrap();
		assert!(plan2.pending_migrations.is_empty());
		assert_eq!(plan2.schema_modules.len(), 2);
	});
}

#[test]
fn idempotent_apply() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("idempotent_apply");

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		let result = plan.apply(&db).await.unwrap();
		assert_eq!(result.applied_migrations, 2);

		// Second apply — only schema re-applied
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		assert!(plan.pending_migrations.is_empty());
		let result = plan.apply(&db).await.unwrap();
		assert_eq!(result.applied_migrations, 0);
		assert_eq!(result.applied_modules, 2);
	});
}

#[test]
fn triple_apply_still_idempotent() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("triple_apply");

	rt.block_on(async {
		for i in 0..3 {
			let plan = overshift::plan(&db, &manifest).await.unwrap();
			let result = plan.apply(&db).await.unwrap();
			if i == 0 {
				assert_eq!(result.applied_migrations, 2);
			} else {
				assert_eq!(result.applied_migrations, 0);
			}
			assert_eq!(result.applied_modules, 2);
		}
	});
}

// ─── Dry-run ───

#[test]
fn plan_dry_run_does_not_modify() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("plan_dry_run");

	rt.block_on(async {
		let plan1 = overshift::plan(&db, &manifest).await.unwrap();
		assert_eq!(plan1.pending_migrations.len(), 2);

		// Plan again — still same pending
		let plan2 = overshift::plan(&db, &manifest).await.unwrap();
		assert_eq!(plan2.pending_migrations.len(), 2);
	});
}

#[test]
fn plan_print_does_not_panic() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("plan_print");

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		// Just ensure print() doesn't panic
		plan.print();
	});
}

#[test]
fn plan_is_empty_when_nothing_to_do() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("plan_is_empty");

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		assert!(!plan.is_empty());

		plan.apply(&db).await.unwrap();

		// After apply, only schema remains (always re-applied)
		let plan2 = overshift::plan(&db, &manifest).await.unwrap();
		// pending_migrations is empty, but schema_modules is not
		assert!(plan2.pending_migrations.is_empty());
		assert!(!plan2.schema_modules.is_empty());
	});
}

// ─── Changelog ───

#[test]
fn apply_records_changelog() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("changelog");

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		plan.apply(&db).await.unwrap();

		db.use_ns(&manifest.meta.ns)
			.use_db(&manifest.meta.system_db)
			.await
			.unwrap();

		let mut response = db
			.query("SELECT * FROM changelog ORDER BY name")
			.await
			.unwrap();
		let rows: Vec<serde_json::Value> = response.take(0).unwrap();

		// 2 migrations + 2 schema modules = 4 entries
		assert_eq!(rows.len(), 4);

		let types: Vec<&str> = rows
			.iter()
			.filter_map(|r| r.get("type").and_then(|t| t.as_str()))
			.collect();
		assert!(types.contains(&"migration"));
		assert!(types.contains(&"schema_module"));
	});
}

#[test]
fn changelog_grows_on_reapply() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("changelog_grows");

	rt.block_on(async {
		// First apply
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		plan.apply(&db).await.unwrap();

		// Second apply (only schema)
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		plan.apply(&db).await.unwrap();

		db.use_ns(&manifest.meta.ns)
			.use_db(&manifest.meta.system_db)
			.await
			.unwrap();

		let mut response = db.query("SELECT * FROM changelog").await.unwrap();
		let rows: Vec<serde_json::Value> = response.take(0).unwrap();

		// 4 from first apply + 2 schema from second apply = 6
		assert_eq!(rows.len(), 6);
	});
}

#[test]
fn changelog_entry_has_all_fields() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("changelog_fields");

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		let result = plan.apply(&db).await.unwrap();

		db.use_ns(&manifest.meta.ns)
			.use_db(&manifest.meta.system_db)
			.await
			.unwrap();

		let mut response = db
			.query("SELECT * FROM changelog WHERE type = 'migration' ORDER BY version LIMIT 1")
			.await
			.unwrap();
		let rows: Vec<serde_json::Value> = response.take(0).unwrap();
		assert_eq!(rows.len(), 1);

		let entry = &rows[0];
		assert!(entry.get("version").is_some(), "missing version");
		assert!(entry.get("name").is_some(), "missing name");
		assert!(entry.get("type").is_some(), "missing type");
		assert!(entry.get("checksum").is_some(), "missing checksum");
		assert!(entry.get("applied_at").is_some(), "missing applied_at");
		assert!(entry.get("applied_by").is_some(), "missing applied_by");
		assert_eq!(
			entry.get("applied_by").and_then(|v| v.as_str()),
			Some(result.instance_id.as_str()),
		);
	});
}

// ─── Function validation ───

#[test]
fn validate_functions_passes_after_apply() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("validate_fns_ok");

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		let expected_fns = plan.functions_to_validate.clone();
		assert!(!expected_fns.is_empty());

		plan.apply(&db).await.unwrap();

		db.use_ns(&manifest.meta.ns)
			.use_db(&manifest.meta.db)
			.await
			.unwrap();

		// Should succeed — all functions exist
		overshift::validate::validate_functions(&db, &expected_fns)
			.await
			.unwrap();
	});
}

#[test]
fn validate_functions_fails_for_missing() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("validate_fns_missing");

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		plan.apply(&db).await.unwrap();

		db.use_ns(&manifest.meta.ns)
			.use_db(&manifest.meta.db)
			.await
			.unwrap();

		let bogus = vec!["nonexistent_function".to_string()];
		let result = overshift::validate::validate_functions(&db, &bogus).await;
		assert!(result.is_err());
		let err = result.unwrap_err().to_string();
		assert!(
			err.contains("nonexistent_function"),
			"expected function name in error, got: {err}"
		);
	});
}

#[test]
fn validate_functions_exact_match_no_false_positives() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("validate_fns_exact");

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		plan.apply(&db).await.unwrap();

		db.use_ns(&manifest.meta.ns)
			.use_db(&manifest.meta.db)
			.await
			.unwrap();

		// "greet" exists, but "eet" should NOT match via ends_with
		let partial = vec!["eet".to_string()];
		let result = overshift::validate::validate_functions(&db, &partial).await;
		assert!(result.is_err(), "partial name should not match");
	});
}

// ─── Schema verification ───

#[test]
fn schema_tables_exist_after_apply() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("schema_tables");

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		plan.apply(&db).await.unwrap();

		db.use_ns(&manifest.meta.ns)
			.use_db(&manifest.meta.db)
			.await
			.unwrap();

		let mut response = db.query("INFO FOR DB").await.unwrap();
		let info: Option<serde_json::Value> = response.take(0).unwrap();
		let info = info.unwrap();

		let tables = info.get("tables").and_then(|t| t.as_object()).unwrap();
		assert!(tables.contains_key("user"));
		assert!(tables.contains_key("post"));

		let functions = info.get("functions").and_then(|f| f.as_object()).unwrap();
		assert!(!functions.is_empty());
	});
}

#[test]
fn schema_indexes_exist_after_apply() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("schema_indexes");

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		plan.apply(&db).await.unwrap();

		db.use_ns(&manifest.meta.ns)
			.use_db(&manifest.meta.db)
			.await
			.unwrap();

		let mut response = db.query("INFO FOR TABLE user").await.unwrap();
		let info: Option<serde_json::Value> = response.take(0).unwrap();
		let info = info.unwrap();

		let indexes = info.get("indexes").and_then(|i| i.as_object()).unwrap();
		assert!(indexes.contains_key("idx_user_email"));
	});
}

#[test]
fn functions_callable_after_apply() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("functions_callable");

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		plan.apply(&db).await.unwrap();

		db.use_ns(&manifest.meta.ns)
			.use_db(&manifest.meta.db)
			.await
			.unwrap();

		// Call fn::greet
		let mut response = db.query("RETURN fn::greet('World')").await.unwrap();
		let result: Option<String> = response.take(0).unwrap();
		assert_eq!(result, Some("Hello, World!".to_string()));
	});
}

// ─── Migration data ───

#[test]
fn migration_data_persists() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("migration_data");

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		plan.apply(&db).await.unwrap();

		db.use_ns(&manifest.meta.ns)
			.use_db(&manifest.meta.db)
			.await
			.unwrap();

		let mut response = db.query("SELECT * FROM user").await.unwrap();
		let users: Vec<serde_json::Value> = response.take(0).unwrap();
		assert!(users.len() >= 2, "expected at least 2 users from seed");
	});
}

// ─── Migration lock tracking ───

#[test]
fn lock_tracks_applied_migrations() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("lock_tracking");

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		let result = plan.apply(&db).await.unwrap();

		db.use_ns(&manifest.meta.ns)
			.use_db(&manifest.meta.system_db)
			.await
			.unwrap();

		let mut response = db
			.query("SELECT * FROM migration_lock ORDER BY version")
			.await
			.unwrap();
		let locks: Vec<serde_json::Value> = response.take(0).unwrap();

		assert_eq!(locks.len(), 2);

		let v1 = &locks[0];
		assert_eq!(v1.get("version").and_then(|v| v.as_i64()), Some(1));
		assert!(v1.get("checksum").and_then(|v| v.as_str()).is_some());
		assert_eq!(
			v1.get("instance_id").and_then(|v| v.as_str()),
			Some(result.instance_id.as_str())
		);
	});
}

#[test]
fn lock_released_after_apply() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("lock_released");

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		plan.apply(&db).await.unwrap();

		// Verify lock is released
		db.use_ns(&manifest.meta.ns)
			.use_db(&manifest.meta.system_db)
			.await
			.unwrap();

		let mut response = db
			.query("SELECT * FROM leader_lock:migration")
			.await
			.unwrap();
		let rows: Vec<serde_json::Value> = response.take(0).unwrap();

		if let Some(leader) = rows.first() {
			// holder should be NONE (null) after release
			let holder = leader.get("holder");
			assert!(
				holder.is_none() || holder == Some(&serde_json::Value::Null),
				"lock should be released, got holder: {holder:?}"
			);
		}
	});
}

// ─── System tables bootstrap ───

#[test]
fn system_tables_bootstrapped() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("bootstrap");

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		plan.apply(&db).await.unwrap();

		db.use_ns(&manifest.meta.ns)
			.use_db(&manifest.meta.system_db)
			.await
			.unwrap();

		let mut response = db.query("INFO FOR DB").await.unwrap();
		let info: Option<serde_json::Value> = response.take(0).unwrap();
		let info = info.unwrap();

		let tables = info.get("tables").and_then(|t| t.as_object()).unwrap();
		assert!(tables.contains_key("migration_lock"));
		assert!(tables.contains_key("leader_lock"));
		assert!(tables.contains_key("shedlock"));
		assert!(tables.contains_key("changelog"));
	});
}

// ─── Edge cases ───

#[test]
fn apply_with_no_migrations_dir() {
	let rt = runtime();
	let db = connect();

	let tmp = tempfile::tempdir().unwrap();
	let mod_dir = tmp.path().join("schema/core");
	std::fs::create_dir_all(&mod_dir).unwrap();
	std::fs::write(
		mod_dir.join("table.surql"),
		"DEFINE TABLE OVERWRITE item SCHEMAFULL;",
	)
	.unwrap();
	std::fs::write(
		tmp.path().join("manifest.toml"),
		r#"
		[meta]
		ns = "test_no_mig"
		db = "main"
		system_db = "_system"

		[[modules]]
		name = "core"
		path = "schema/core"
	"#,
	)
	.unwrap();

	let manifest = Manifest::load(tmp.path()).unwrap();

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		assert!(plan.pending_migrations.is_empty());
		assert_eq!(plan.schema_modules.len(), 1);

		let result = plan.apply(&db).await.unwrap();
		assert_eq!(result.applied_migrations, 0);
		assert_eq!(result.applied_modules, 1);
	});
}

#[test]
fn apply_with_no_schema_modules() {
	let rt = runtime();
	let db = connect();

	let tmp = tempfile::tempdir().unwrap();
	let mig_dir = tmp.path().join("migrations");
	std::fs::create_dir(&mig_dir).unwrap();
	std::fs::write(
		mig_dir.join("v001_init.surql"),
		"DEFINE TABLE IF NOT EXISTS standalone SCHEMAFULL;",
	)
	.unwrap();
	std::fs::write(
		tmp.path().join("manifest.toml"),
		r#"
		[meta]
		ns = "test_no_schema"
		db = "main"
		system_db = "_system"
	"#,
	)
	.unwrap();

	let manifest = Manifest::load(tmp.path()).unwrap();

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		assert_eq!(plan.pending_migrations.len(), 1);
		assert!(plan.schema_modules.is_empty());

		let result = plan.apply(&db).await.unwrap();
		assert_eq!(result.applied_migrations, 1);
		assert_eq!(result.applied_modules, 0);
	});
}

#[test]
fn apply_empty_project() {
	let rt = runtime();
	let db = connect();

	let tmp = tempfile::tempdir().unwrap();
	std::fs::write(
		tmp.path().join("manifest.toml"),
		r#"
		[meta]
		ns = "test_empty_proj"
		db = "main"
		system_db = "_system"
	"#,
	)
	.unwrap();

	let manifest = Manifest::load(tmp.path()).unwrap();

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		assert!(plan.is_empty());

		let result = plan.apply(&db).await.unwrap();
		assert_eq!(result.applied_migrations, 0);
		assert_eq!(result.applied_modules, 0);
	});
}

#[test]
fn checksum_mismatch_detected() {
	let rt = runtime();
	let db = connect();

	let tmp = tempfile::tempdir().unwrap();
	let mig_dir = tmp.path().join("migrations");
	std::fs::create_dir(&mig_dir).unwrap();
	std::fs::write(mig_dir.join("v001_init.surql"), "-- original content").unwrap();
	std::fs::write(
		tmp.path().join("manifest.toml"),
		r#"
		[meta]
		ns = "test_checksum"
		db = "main"
		system_db = "_system"
	"#,
	)
	.unwrap();

	let manifest = Manifest::load(tmp.path()).unwrap();

	rt.block_on(async {
		// Apply first version
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		plan.apply(&db).await.unwrap();

		// Modify the migration file (forbidden!)
		std::fs::write(mig_dir.join("v001_init.surql"), "-- MODIFIED content").unwrap();

		// Plan should detect checksum mismatch
		let result = overshift::plan(&db, &manifest).await;
		assert!(result.is_err());
		let err = result.unwrap_err().to_string();
		assert!(
			err.contains("checksum mismatch"),
			"expected checksum error, got: {err}"
		);
	});
}

#[test]
fn different_instances_get_different_ids() {
	let rt = runtime();
	let db = connect();
	let manifest = test_manifest("diff_instances");

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		let r1 = plan.apply(&db).await.unwrap();

		// Second apply (different instance ID)
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		let r2 = plan.apply(&db).await.unwrap();

		assert_ne!(r1.instance_id, r2.instance_id);
	});
}

#[test]
fn should_reject_duplicate_in_plan() {
	let rt = runtime();
	let db = connect();

	let tmp = tempfile::tempdir().unwrap();
	let mig_dir = tmp.path().join("migrations");
	std::fs::create_dir(&mig_dir).unwrap();

	std::fs::write(mig_dir.join("v001_init.surql"), "-- init").unwrap();
	std::fs::write(mig_dir.join("v002_seed.surql"), "-- seed v1").unwrap();
	std::fs::write(mig_dir.join("v003_first.surql"), "-- first").unwrap();
	std::fs::write(
		tmp.path().join("manifest.toml"),
		r#"
		[meta]
		ns = "test_dup_plan"
		db = "main"
		system_db = "_system"
	"#,
	)
	.unwrap();

	let manifest = Manifest::load(tmp.path()).unwrap();

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		plan.apply(&db).await.unwrap();

		// Modify v002 content after it was already applied — checksum mismatch
		std::fs::write(mig_dir.join("v002_seed.surql"), "-- seed v2 MODIFIED").unwrap();

		let result = overshift::plan(&db, &manifest).await;
		assert!(
			result.is_err(),
			"should detect checksum mismatch for already-applied v002"
		);
		let err = result.unwrap_err().to_string();
		assert!(
			err.contains("checksum mismatch"),
			"expected checksum mismatch error, got: {err}"
		);
	});
}
