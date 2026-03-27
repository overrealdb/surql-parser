//! Plan and apply — the core migration engine workflow.
//!
//! `plan()` examines the filesystem and database to determine what actions are
//! needed. `Plan::apply()` executes those actions with distributed locking.

use std::collections::HashSet;

use surql_macros::surql_check;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use tracing::{debug, error, info};

use crate::Error;
use crate::changelog;
use crate::lock::SurrealLock;
use crate::manifest::{Manifest, ManifestMeta};
use crate::migration::{AppliedMigration, Migration, compute_checksum, discover_migrations};
use crate::schema::{SchemaModule, extract_function_names, load_schema_modules};
use crate::validate;

/// Bootstrap SQL for _system tables.
const BOOTSTRAP_SQL: &str = include_str!("../sql/bootstrap.surql");

/// Query: record a successfully applied migration in `migration_lock`.
const RECORD_MIGRATION_SQL: &str = surql_check!(
	r#"
	CREATE migration_lock SET
		version = $version,
		applied_at = time::now(),
		checksum = $checksum,
		instance_id = $instance_id
"#
);

/// Query: read all applied migrations from `migration_lock`.
const READ_APPLIED_SQL: &str = surql_check!(
	r#"
	SELECT
		version,
		<string> applied_at AS applied_at,
		checksum,
		instance_id
	FROM migration_lock
	ORDER BY version ASC
"#
);

/// Query: delete a migration from `migration_lock` by version.
const DELETE_MIGRATION_SQL: &str = surql_check!(
	r#"
	DELETE FROM migration_lock WHERE version = $version
"#
);

/// The result of rolling back migrations.
#[derive(Debug)]
pub struct RollbackResult {
	pub rolled_back: Vec<String>,
	pub target_version: u32,
	pub errors: Vec<String>,
}

/// The result of `plan()` — everything needed to apply changes.
#[derive(Debug)]
pub struct Plan {
	pub meta: ManifestMeta,
	pub pending_migrations: Vec<Migration>,
	pub schema_modules: Vec<SchemaModule>,
	pub functions_to_validate: Vec<String>,
	#[cfg(feature = "shadow")]
	pub(crate) manifest: Manifest,
}

/// The result of `Plan::apply()`.
#[derive(Debug)]
pub struct ApplyResult {
	pub applied_migrations: usize,
	pub applied_modules: usize,
	pub instance_id: String,
	#[cfg(feature = "shadow")]
	pub schema_drift: Option<String>,
}

impl Plan {
	/// Returns `true` if there are no pending migrations and no schema to apply.
	pub fn is_empty(&self) -> bool {
		self.pending_migrations.is_empty() && self.schema_modules.is_empty()
	}

	/// Print a human-readable summary of what will be done.
	pub fn print(&self) {
		if self.is_empty() {
			println!("Nothing to do — database is up to date.");
			return;
		}

		if !self.pending_migrations.is_empty() {
			println!("Pending migrations:");
			for m in &self.pending_migrations {
				println!("  v{:03}_{} ({})", m.version, m.name, &m.checksum[..8]);
			}
		}

		if !self.schema_modules.is_empty() {
			println!("\nSchema modules to apply:");
			for m in &self.schema_modules {
				println!(
					"  {} ({} bytes, {} files)",
					m.name,
					m.content.len(),
					m.files.len()
				);
			}
		}

		if !self.functions_to_validate.is_empty() {
			println!("\nFunctions to validate:");
			for f in &self.functions_to_validate {
				println!("  fn::{f}");
			}
		}
	}

	/// Apply the plan: run pending migrations, apply schema modules, validate.
	///
	/// This acquires a distributed lock, applies all changes, records changelog
	/// entries, and releases the lock. The DB connection is left pointing at the
	/// main application database.
	pub async fn apply(self, db: &Surreal<Any>) -> crate::Result<ApplyResult> {
		let instance_id = uuid::Uuid::new_v4().to_string();

		if self.is_empty() {
			info!("nothing to apply — database is up to date");
			// Ensure DB is pointing at main
			db.use_ns(&self.meta.ns).use_db(&self.meta.db).await?;
			return Ok(ApplyResult {
				applied_migrations: 0,
				applied_modules: 0,
				instance_id,
				#[cfg(feature = "shadow")]
				schema_drift: None,
			});
		}

		info!(
			pending_migrations = self.pending_migrations.len(),
			schema_modules = self.schema_modules.len(),
			instance_id = %instance_id,
			"starting apply"
		);

		// Switch to _system DB for lock operations
		db.use_ns(&self.meta.ns)
			.use_db(&self.meta.system_db)
			.await?;

		// Bootstrap _system tables (idempotent)
		db.query(BOOTSTRAP_SQL)
			.await
			.map_err(|e| Error::Migration(format!("bootstrap failed: {e}")))?;

		// Acquire distributed lock
		let lock = SurrealLock::new(db.clone(), instance_id.clone(), "migration")?;
		lock.acquire().await?;

		// Apply everything, always releasing the lock afterward
		#[cfg(feature = "shadow")]
		let manifest = self.manifest.clone();
		let result = apply_inner(db, &self, &instance_id).await;

		// Post-apply verification: compare real DB against shadow DB
		#[cfg(feature = "shadow")]
		let schema_drift = if result.is_ok() {
			verify_against_shadow_after_apply(db, &self.meta, &manifest).await
		} else {
			None
		};

		if let Err(e) = lock.release().await {
			error!("failed to release migration lock: {e}");
		}

		let (applied_migrations, applied_modules) = result?;

		// Leave DB pointing at the main application database
		db.use_ns(&self.meta.ns).use_db(&self.meta.db).await?;

		info!(applied_migrations, applied_modules, "apply complete");

		Ok(ApplyResult {
			applied_migrations,
			applied_modules,
			instance_id,
			#[cfg(feature = "shadow")]
			schema_drift,
		})
	}
}

/// Internal apply logic — separated so we can always release the lock.
async fn apply_inner(
	db: &Surreal<Any>,
	plan: &Plan,
	instance_id: &str,
) -> crate::Result<(usize, usize)> {
	let mut applied_migrations = 0;
	let mut migration_errors: Vec<String> = Vec::new();

	// Apply pending migrations (each wrapped in its own transaction)
	for migration in &plan.pending_migrations {
		db.use_ns(&plan.meta.ns).use_db(&plan.meta.db).await?;

		info!(
			version = migration.version,
			name = %migration.name,
			"applying migration"
		);

		let wrapped = format!(
			"BEGIN TRANSACTION;\n{}\nCOMMIT TRANSACTION;",
			migration.content
		);

		let migration_result = match db.query(&wrapped).await {
			Ok(response) => match response.check() {
				Ok(_) => Ok(()),
				Err(e) => Err(format!(
					"migration v{:03}_{} had errors: {e}",
					migration.version, migration.name,
				)),
			},
			Err(e) => Err(format!(
				"migration v{:03}_{} failed: {e}",
				migration.version, migration.name,
			)),
		};

		db.use_ns(&plan.meta.ns)
			.use_db(&plan.meta.system_db)
			.await?;

		match migration_result {
			Ok(()) => {
				record_migration(db, migration, instance_id).await?;

				changelog::record_entry(
					db,
					"migration",
					migration.version,
					&migration.name,
					&migration.checksum,
					instance_id,
				)
				.await?;

				applied_migrations += 1;

				info!(
					version = migration.version,
					name = %migration.name,
					"migration applied successfully"
				);
			}
			Err(err_msg) => {
				error!(
					version = migration.version,
					name = %migration.name,
					error = %err_msg,
					"migration failed, stopping — subsequent migrations depend on prior state"
				);

				changelog::record_entry(
					db,
					"migration_failed",
					migration.version,
					&migration.name,
					&migration.checksum,
					instance_id,
				)
				.await?;

				migration_errors.push(err_msg);
				break;
			}
		}
	}

	// Apply schema modules (no transaction — DEFINE OVERWRITE is idempotent)
	db.use_ns(&plan.meta.ns).use_db(&plan.meta.db).await?;
	let mut applied_modules = 0;

	for module in &plan.schema_modules {
		info!(name = %module.name, "applying schema module");

		let response = db
			.query(&module.content)
			.await
			.map_err(|e| Error::Schema(format!("schema module '{}' failed: {e}", module.name)))?;

		response.check().map_err(|e| {
			Error::Schema(format!("schema module '{}' had errors: {e}", module.name))
		})?;

		applied_modules += 1;

		info!(name = %module.name, "schema module applied");
	}

	// Record schema changelog entries
	db.use_ns(&plan.meta.ns)
		.use_db(&plan.meta.system_db)
		.await?;
	for module in &plan.schema_modules {
		let checksum = compute_checksum(&module.content);
		changelog::record_entry(db, "schema_module", 0, &module.name, &checksum, instance_id)
			.await?;
	}

	// Validate functions
	if !plan.functions_to_validate.is_empty() {
		db.use_ns(&plan.meta.ns).use_db(&plan.meta.db).await?;
		validate::validate_functions(db, &plan.functions_to_validate).await?;
	}

	if !migration_errors.is_empty() {
		return Err(Error::Migration(format!(
			"{} migration(s) failed:\n{}",
			migration_errors.len(),
			migration_errors.join("\n")
		)));
	}

	Ok((applied_migrations, applied_modules))
}

/// Record a successfully applied migration in `migration_lock`.
async fn record_migration(
	db: &Surreal<Any>,
	migration: &Migration,
	instance_id: &str,
) -> crate::Result<()> {
	db.query(RECORD_MIGRATION_SQL)
		.bind(("version", migration.version as i64))
		.bind(("checksum", migration.checksum.clone()))
		.bind(("instance_id", instance_id.to_string()))
		.await
		.map_err(|e| {
			Error::Migration(format!(
				"failed to record migration v{:03}_{}: {e}",
				migration.version, migration.name,
			))
		})?;

	Ok(())
}

/// Read which migrations have already been applied from `_system.migration_lock`.
async fn read_applied(db: &Surreal<Any>) -> crate::Result<Vec<AppliedMigration>> {
	let mut response = db
		.query(READ_APPLIED_SQL)
		.await
		.map_err(|e| Error::Migration(format!("read applied migrations failed: {e}")))?;

	let rows: Vec<serde_json::Value> = response
		.take(0)
		.map_err(|e| Error::Migration(format!("take applied migrations failed: {e}")))?;

	let mut applied = Vec::with_capacity(rows.len());
	for row in rows {
		let version = row
			.get("version")
			.and_then(|v| v.as_u64())
			.ok_or_else(|| Error::Migration("corrupt migration_lock: missing version".into()))?
			as u32;
		let applied_at = row
			.get("applied_at")
			.and_then(|v| v.as_str())
			.ok_or_else(|| Error::Migration("corrupt migration_lock: missing applied_at".into()))?
			.to_string();
		let checksum = row
			.get("checksum")
			.and_then(|v| v.as_str())
			.ok_or_else(|| Error::Migration("corrupt migration_lock: missing checksum".into()))?
			.to_string();
		let instance_id = row
			.get("instance_id")
			.and_then(|v| v.as_str())
			.ok_or_else(|| Error::Migration("corrupt migration_lock: missing instance_id".into()))?
			.to_string();

		applied.push(AppliedMigration {
			version,
			applied_at,
			checksum,
			instance_id,
		});
	}

	Ok(applied)
}

/// Examine migrations/schema and the database, return a plan of what will be done.
///
/// For filesystem manifests, reads migration files from `{root}/migrations/` and
/// loads schema modules from disk. For embedded manifests (built via
/// [`Manifest::builder()`]), uses pre-loaded data directly.
///
/// **Note**: This changes the DB connection to point at `_system`. Call
/// `db.use_ns(...).use_db(...)` afterward if you need a different context.
pub async fn plan(db: &Surreal<Any>, manifest: &Manifest) -> crate::Result<Plan> {
	// 1. Load migrations and schema (from preloaded data or filesystem)
	let all_migrations = match &manifest.preloaded_migrations {
		Some(m) => m.clone(),
		None => discover_migrations(manifest.root_path()?)?,
	};
	let schema_modules = match &manifest.preloaded_modules {
		Some(m) => m.clone(),
		None => load_schema_modules(manifest)?,
	};
	let functions_to_validate = extract_function_names(&schema_modules)?;

	info!(
		migrations = all_migrations.len(),
		modules = schema_modules.len(),
		functions = functions_to_validate.len(),
		"discovered schema artifacts"
	);

	// 2. Bootstrap _system and read applied migrations
	db.use_ns(&manifest.meta.ns)
		.use_db(&manifest.meta.system_db)
		.await?;

	// Bootstrap is idempotent — safe to run every time
	let _ = db.query(BOOTSTRAP_SQL).await;

	let applied = read_applied(db).await?;

	debug!(
		applied = applied.len(),
		total = all_migrations.len(),
		"read applied migrations"
	);

	// 3. Validate checksums of already-applied migrations
	for existing in &applied {
		if let Some(expected) = all_migrations
			.iter()
			.find(|m| m.version == existing.version)
			&& existing.checksum != expected.checksum
		{
			return Err(Error::ChecksumMismatch {
				version: existing.version,
				name: expected.name.clone(),
				expected: expected.checksum.clone(),
				actual: existing.checksum.clone(),
			});
		}
	}

	// 4. Detect duplicate versions between applied and filesystem migrations
	//    (e.g., two different migrations both claiming to be v003)
	let applied_versions: HashSet<u32> = applied.iter().map(|a| a.version).collect();
	let filesystem_versions: HashSet<u32> = all_migrations.iter().map(|m| m.version).collect();
	let overlapping: Vec<u32> = applied_versions
		.intersection(&filesystem_versions)
		.copied()
		.collect();

	for version in &overlapping {
		if let (Some(applied_entry), Some(fs_entry)) = (
			applied.iter().find(|a| a.version == *version),
			all_migrations.iter().find(|m| m.version == *version),
		) && applied_entry.checksum != fs_entry.checksum
		{
			return Err(Error::ChecksumMismatch {
				version: *version,
				name: fs_entry.name.clone(),
				expected: fs_entry.checksum.clone(),
				actual: applied_entry.checksum.clone(),
			});
		}
	}

	// Check for duplicate versions among pending migrations themselves
	{
		let mut seen = HashSet::new();
		for m in &all_migrations {
			if !seen.insert(m.version) {
				return Err(Error::Migration(format!(
					"duplicate migration version in plan: v{:03}",
					m.version,
				)));
			}
		}
	}

	// 5. Compute pending migrations
	let pending_migrations: Vec<Migration> = all_migrations
		.into_iter()
		.filter(|m| !applied_versions.contains(&m.version))
		.collect();

	Ok(Plan {
		meta: manifest.meta.clone(),
		pending_migrations,
		schema_modules,
		functions_to_validate,
		#[cfg(feature = "shadow")]
		manifest: manifest.clone(),
	})
}

/// Roll back applied migrations to a target version.
///
/// Finds all applied migrations with version > `target_version`, executes their
/// `.down.surql` content in reverse order, removes them from `migration_lock`,
/// and records each rollback in the changelog.
///
/// Returns an error in `RollbackResult::errors` for any migration that lacks a
/// `.down.surql` companion file — those migrations are skipped (not rolled back).
pub async fn rollback(
	db: &Surreal<Any>,
	manifest: &Manifest,
	target_version: u32,
) -> crate::Result<RollbackResult> {
	let instance_id = uuid::Uuid::new_v4().to_string();

	// Load all known migrations (from preloaded data or filesystem)
	let all_migrations = match &manifest.preloaded_migrations {
		Some(m) => m.clone(),
		None => discover_migrations(manifest.root_path()?)?,
	};

	// Switch to _system DB to read applied migrations
	db.use_ns(&manifest.meta.ns)
		.use_db(&manifest.meta.system_db)
		.await?;

	// Bootstrap is idempotent — safe to run every time
	let _ = db.query(BOOTSTRAP_SQL).await;

	let applied = read_applied(db).await?;

	// Find applied migrations with version > target that need rollback
	let mut to_rollback: Vec<&AppliedMigration> = applied
		.iter()
		.filter(|a| a.version > target_version)
		.collect();

	// Roll back in reverse version order
	to_rollback.sort_by(|a, b| b.version.cmp(&a.version));

	if to_rollback.is_empty() {
		info!(
			target_version,
			"nothing to roll back — no migrations above target version"
		);
		db.use_ns(&manifest.meta.ns)
			.use_db(&manifest.meta.db)
			.await?;
		return Ok(RollbackResult {
			rolled_back: Vec::new(),
			target_version,
			errors: Vec::new(),
		});
	}

	// Acquire distributed lock
	let lock = SurrealLock::new(db.clone(), instance_id.clone(), "migration")?;
	lock.acquire().await?;

	let mut rolled_back = Vec::new();
	let mut errors = Vec::new();

	for applied_mig in &to_rollback {
		let migration = all_migrations
			.iter()
			.find(|m| m.version == applied_mig.version);

		let down_content = migration.and_then(|m| m.down_content.as_deref());

		match down_content {
			None => {
				let msg = format!(
					"v{:03}: no .down.surql file — cannot roll back",
					applied_mig.version,
				);
				error!(version = applied_mig.version, "{msg}");
				errors.push(msg);
				continue;
			}
			Some(sql) => {
				// Execute the down migration against the application DB
				db.use_ns(&manifest.meta.ns)
					.use_db(&manifest.meta.db)
					.await?;

				let wrapped = format!("BEGIN TRANSACTION;\n{}\nCOMMIT TRANSACTION;", sql);

				let exec_result = match db.query(&wrapped).await {
					Ok(response) => match response.check() {
						Ok(_) => Ok(()),
						Err(e) => Err(format!(
							"v{:03}: down migration had errors: {e}",
							applied_mig.version,
						)),
					},
					Err(e) => Err(format!(
						"v{:03}: down migration failed: {e}",
						applied_mig.version,
					)),
				};

				// Switch back to _system for bookkeeping
				db.use_ns(&manifest.meta.ns)
					.use_db(&manifest.meta.system_db)
					.await?;

				match exec_result {
					Ok(()) => {
						// Remove from migration_lock
						db.query(DELETE_MIGRATION_SQL)
							.bind(("version", applied_mig.version as i64))
							.await
							.map_err(|e| {
								Error::Migration(format!(
									"failed to remove v{:03} from migration_lock: {e}",
									applied_mig.version,
								))
							})?;

						let name = migration.map(|m| m.name.as_str()).unwrap_or("unknown");

						changelog::record_entry(
							db,
							"rollback",
							applied_mig.version,
							name,
							&applied_mig.checksum,
							&instance_id,
						)
						.await?;

						let label = format!("v{:03}_{}", applied_mig.version, name);
						info!(version = applied_mig.version, name, "rolled back");
						rolled_back.push(label);
					}
					Err(err_msg) => {
						error!(
							version = applied_mig.version,
							error = %err_msg,
							"rollback failed, stopping — further rollbacks would leave DB in worse state"
						);
						errors.push(err_msg);
						break;
					}
				}
			}
		}
	}

	if let Err(e) = lock.release().await {
		error!("failed to release migration lock: {e}");
	}

	// Leave DB pointing at the main application database
	db.use_ns(&manifest.meta.ns)
		.use_db(&manifest.meta.db)
		.await?;

	info!(
		rolled_back = rolled_back.len(),
		errors = errors.len(),
		target_version,
		"rollback complete"
	);

	Ok(RollbackResult {
		rolled_back,
		target_version,
		errors,
	})
}

/// Post-apply verification: apply the manifest to a shadow DB and compare with the real DB.
///
/// Returns `Some(diff_string)` if schema drift is detected, `None` if schemas match.
/// This is warning-only — the migrations already applied successfully.
#[cfg(feature = "shadow")]
async fn verify_against_shadow_after_apply(
	db: &Surreal<Any>,
	meta: &ManifestMeta,
	manifest: &Manifest,
) -> Option<String> {
	use tracing::warn;

	db.use_ns(&meta.ns).use_db(&meta.db).await.ok()?;

	let real_db_info = match validate::query_db_info(db).await {
		Ok(info) => info,
		Err(e) => {
			warn!("post-apply verification: failed to query real DB info: {e}");
			return None;
		}
	};

	let shadow = match crate::shadow::apply_to_shadow(manifest).await {
		Ok(s) => s,
		Err(e) => {
			warn!("post-apply verification: shadow apply failed: {e}");
			return None;
		}
	};

	let diff = validate::compare_db_info(&shadow.db_info, &real_db_info);
	if diff.is_empty() {
		info!("post-apply verification: schema matches shadow DB");
		None
	} else {
		let drift = format!("{diff}");
		warn!("post-apply verification: schema drift detected:\n{drift}");
		Some(drift)
	}
}
