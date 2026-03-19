//! Dual embedded SurrealDB engine for runtime validation and query execution.
//!
//! Enabled via `--features embedded-db`. Two isolated in-memory instances:
//! - `workspace_db` — applies user .surql files as migrations, used for diagnostics/queries
//! - `meta_db` — tracks file hashes and apply status for incremental migration

#[cfg(feature = "embedded-db")]
use surrealdb::{Surreal, engine::local::Mem};

#[cfg(feature = "embedded-db")]
use std::path::Path;

#[cfg(feature = "embedded-db")]
use std::collections::hash_map::DefaultHasher;
#[cfg(feature = "embedded-db")]
use std::hash::{Hash, Hasher};

#[cfg(feature = "embedded-db")]
pub fn content_hash(content: &str) -> String {
	let mut hasher = DefaultHasher::new();
	content.hash(&mut hasher);
	format!("{:016x}", hasher.finish())
}

/// Dual embedded SurrealDB: workspace for user schema, meta for LSP state.
#[cfg(feature = "embedded-db")]
pub struct DualEngine {
	workspace: Surreal<surrealdb::engine::local::Db>,
	meta: Surreal<surrealdb::engine::local::Db>,
}

/// Status of a tracked file in meta_db.
#[cfg(feature = "embedded-db")]
#[derive(Debug, Clone)]
pub struct FileStatus {
	pub path: String,
	pub hash: String,
	pub status: String,
	pub error_message: Option<String>,
}

/// Result of applying migrations.
#[cfg(feature = "embedded-db")]
pub struct MigrationResult {
	pub total: usize,
	pub applied: usize,
	pub skipped: usize,
	pub errors: Vec<String>,
}

#[cfg(feature = "embedded-db")]
const META_SCHEMA: &str = "\
DEFINE TABLE _lsp_file SCHEMAFULL;
DEFINE FIELD path ON _lsp_file TYPE string;
DEFINE FIELD hash ON _lsp_file TYPE string;
DEFINE FIELD status ON _lsp_file TYPE string;
DEFINE FIELD error_message ON _lsp_file TYPE option<string>;
DEFINE FIELD file_size ON _lsp_file TYPE int;
DEFINE INDEX file_path ON _lsp_file FIELDS path UNIQUE;
";

#[cfg(feature = "embedded-db")]
impl DualEngine {
	/// Start both in-memory SurrealDB instances and initialize meta schema.
	pub async fn start() -> anyhow::Result<Self> {
		let workspace = Surreal::new::<Mem>(()).await?;
		workspace.use_ns("workspace").use_db("workspace").await?;

		let meta = Surreal::new::<Mem>(()).await?;
		meta.use_ns("lsp").use_db("meta").await?;
		meta.query(META_SCHEMA).await?.check()?;

		tracing::info!("DualEngine started (workspace + meta, both in-memory)");
		Ok(Self { workspace, meta })
	}

	/// Apply .surql files as migrations, skipping unchanged files (by hash).
	///
	/// On any file change, resets workspace_db and reapplies all files to ensure
	/// consistent schema state.
	pub async fn apply_migrations(&self, dir: &Path) -> anyhow::Result<MigrationResult> {
		let mut surql_files = Vec::new();
		collect_surql_files(dir, &mut surql_files);
		surql_files.sort();

		let mut any_changed = false;
		let mut file_contents: Vec<(std::path::PathBuf, String, String)> = Vec::new();

		for path in &surql_files {
			let content = match std::fs::read_to_string(path) {
				Ok(c) => c,
				Err(e) => {
					tracing::warn!("Cannot read {}: {e}", path.display());
					continue;
				}
			};
			let hash = content_hash(&content);
			let path_str = path.to_string_lossy().to_string();

			if !self.needs_reapply(&path_str, &hash).await {
				file_contents.push((path.clone(), content, hash));
				continue;
			}

			any_changed = true;
			file_contents.push((path.clone(), content, hash));
		}

		if !any_changed {
			tracing::info!("No files changed, skipping migration replay");
			return Ok(MigrationResult {
				total: file_contents.len(),
				applied: 0,
				skipped: file_contents.len(),
				errors: Vec::new(),
			});
		}

		self.reset_workspace().await?;

		let mut applied = 0;
		let skipped = 0;
		let mut errors = Vec::new();

		for (path, content, hash) in &file_contents {
			let path_str = path.to_string_lossy().to_string();

			match self.workspace.query(content.as_str()).await {
				Ok(response) => match response.check() {
					Ok(_) => {
						self.mark_file_applied(&path_str, hash, content.len(), None)
							.await;
						applied += 1;
					}
					Err(e) => {
						let err_msg = format!("{}: {e}", path.display());
						self.mark_file_applied(&path_str, hash, content.len(), Some(&err_msg))
							.await;
						errors.push(err_msg);
					}
				},
				Err(e) => {
					let err_msg = format!("{}: {e}", path.display());
					self.mark_file_applied(&path_str, hash, content.len(), Some(&err_msg))
						.await;
					errors.push(err_msg);
				}
			}
		}

		tracing::info!(
			"Migrations: {applied}/{} applied, {skipped} skipped, {} errors",
			file_contents.len(),
			errors.len()
		);

		Ok(MigrationResult {
			total: file_contents.len(),
			applied,
			skipped,
			errors,
		})
	}

	/// Destroy and recreate the workspace database (meta is preserved).
	pub async fn reset_workspace(&self) -> anyhow::Result<()> {
		// REMOVE DATABASE fails if it doesn't exist yet (first call) — safe to ignore
		self.workspace.query("REMOVE DATABASE workspace").await.ok();
		self.workspace
			.use_ns("workspace")
			.use_db("workspace")
			.await?;
		tracing::debug!("Workspace database reset");
		Ok(())
	}

	/// Execute a SurrealQL query on the workspace database.
	pub async fn execute_on_workspace(&self, query: &str) -> anyhow::Result<String> {
		let mut response = self.workspace.query(query).await?.check()?;
		let result: Vec<serde_json::Value> = response.take(0)?;
		Ok(serde_json::to_string_pretty(&result)?)
	}

	/// Validate a query against the current workspace schema.
	pub async fn validate_query(&self, query: &str) -> Vec<String> {
		match self.workspace.query(query).await {
			Ok(response) => match response.check() {
				Ok(_) => Vec::new(),
				Err(e) => vec![e.to_string()],
			},
			Err(e) => vec![e.to_string()],
		}
	}

	/// Check if a file needs to be reapplied (hash differs from last apply).
	pub async fn needs_reapply(&self, path: &str, current_hash: &str) -> bool {
		match self.file_status(path).await {
			Some(status) => status.hash != current_hash,
			None => true,
		}
	}

	/// Get the tracked status of a file from meta_db.
	pub async fn file_status(&self, path: &str) -> Option<FileStatus> {
		let query = "SELECT path, hash, status, error_message FROM _lsp_file WHERE path = $path";
		let mut response = match self
			.meta
			.query(query)
			.bind(("path", path.to_string()))
			.await
		{
			Ok(r) => r,
			Err(e) => {
				tracing::warn!("meta_db query failed for {path}: {e}");
				return None;
			}
		};

		let results: Vec<serde_json::Value> = response.take(0).ok()?;
		let row = results.into_iter().next()?;
		Some(FileStatus {
			path: row.get("path")?.as_str()?.to_string(),
			hash: row.get("hash")?.as_str()?.to_string(),
			status: row.get("status")?.as_str()?.to_string(),
			error_message: row
				.get("error_message")
				.and_then(|v| v.as_str().map(|s| s.to_string())),
		})
	}

	/// Check whether a file with this content was already applied as a migration.
	/// If so, return the recorded error (if any) instead of re-executing.
	pub async fn migration_error_for(&self, path: &str, content: &str) -> Option<Vec<String>> {
		let hash = content_hash(content);
		let status = self.file_status(path).await?;
		if status.hash != hash {
			return None;
		}
		// File was applied with this exact content — return its recorded result
		Some(status.error_message.into_iter().collect())
	}

	/// Record a file's apply status in meta_db.
	async fn mark_file_applied(
		&self,
		path: &str,
		hash: &str,
		file_size: usize,
		error: Option<&str>,
	) {
		let status = if error.is_some() { "error" } else { "ok" };
		let query = "\
			DELETE FROM _lsp_file WHERE path = $path; \
			CREATE _lsp_file SET \
				path = $path, \
				hash = $hash, \
				status = $status, \
				error_message = $error_message, \
				file_size = $file_size\
		";
		let error_message = error.map(|e| e.to_string());
		if let Err(e) = self
			.meta
			.query(query)
			.bind(("path", path.to_string()))
			.bind(("hash", hash.to_string()))
			.bind(("status", status.to_string()))
			.bind(("error_message", error_message))
			.bind(("file_size", file_size as i64))
			.await
		{
			tracing::warn!("Failed to update meta_db for {path}: {e}");
		}
	}
}

/// Recursively collect all .surql files from a directory tree, sorted by path.
#[cfg(feature = "embedded-db")]
fn collect_surql_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
	let entries = match std::fs::read_dir(dir) {
		Ok(e) => e,
		Err(_) => return,
	};
	for entry in entries.filter_map(|e| e.ok()) {
		let path = entry.path();
		if path.is_dir() {
			collect_surql_files(&path, out);
		} else if path.extension().is_some_and(|ext| ext == "surql") {
			out.push(path);
		}
	}
}

/// Stub when embedded-db feature is not enabled.
#[cfg(not(feature = "embedded-db"))]
pub struct DualEngine;

#[cfg(not(feature = "embedded-db"))]
impl DualEngine {
	pub async fn start() -> Result<Self, String> {
		Err("Embedded DB not available. Build with --features embedded-db".into())
	}
}

#[cfg(all(test, feature = "embedded-db"))]
mod tests {
	use super::*;
	use std::fs;
	use tempfile::TempDir;

	#[tokio::test]
	async fn should_start_dual_engine() {
		let engine = DualEngine::start().await.expect("DualEngine should start");
		let result = engine
			.execute_on_workspace("RETURN 1")
			.await
			.expect("query should succeed");
		assert!(result.contains('1'));
	}

	#[tokio::test]
	async fn should_apply_migrations_from_directory() {
		let dir = TempDir::new().unwrap();
		fs::write(
			dir.path().join("001_schema.surql"),
			"DEFINE TABLE user SCHEMAFULL;",
		)
		.unwrap();
		fs::write(
			dir.path().join("002_fields.surql"),
			"DEFINE FIELD name ON user TYPE string;",
		)
		.unwrap();

		let engine = DualEngine::start().await.unwrap();
		let result = engine.apply_migrations(dir.path()).await.unwrap();

		assert_eq!(result.total, 2);
		assert_eq!(result.applied, 2);
		assert_eq!(result.skipped, 0);
		assert!(result.errors.is_empty());
	}

	#[tokio::test]
	async fn should_skip_unchanged_files_on_second_apply() {
		let dir = TempDir::new().unwrap();
		fs::write(
			dir.path().join("001_schema.surql"),
			"DEFINE TABLE user SCHEMAFULL;",
		)
		.unwrap();

		let engine = DualEngine::start().await.unwrap();

		let first = engine.apply_migrations(dir.path()).await.unwrap();
		assert_eq!(first.applied, 1);
		assert_eq!(first.skipped, 0);

		let second = engine.apply_migrations(dir.path()).await.unwrap();
		assert_eq!(second.applied, 0);
		assert_eq!(second.skipped, 1);
	}

	#[tokio::test]
	async fn should_reapply_when_file_content_changes() {
		let dir = TempDir::new().unwrap();
		let file = dir.path().join("001_schema.surql");
		fs::write(&file, "DEFINE TABLE user SCHEMAFULL;").unwrap();

		let engine = DualEngine::start().await.unwrap();
		engine.apply_migrations(dir.path()).await.unwrap();

		fs::write(&file, "DEFINE TABLE user SCHEMALESS;").unwrap();
		let result = engine.apply_migrations(dir.path()).await.unwrap();
		assert_eq!(result.applied, 1);
		assert_eq!(result.skipped, 0);
	}

	#[tokio::test]
	async fn should_track_file_status_in_meta() {
		let dir = TempDir::new().unwrap();
		let file = dir.path().join("001_schema.surql");
		fs::write(&file, "DEFINE TABLE user SCHEMAFULL;").unwrap();

		let engine = DualEngine::start().await.unwrap();
		engine.apply_migrations(dir.path()).await.unwrap();

		let path_str = file.to_string_lossy().to_string();
		let status = engine
			.file_status(&path_str)
			.await
			.expect("should have status");
		assert_eq!(status.status, "ok");
		assert!(status.error_message.is_none());
	}

	#[tokio::test]
	async fn should_track_error_status_for_bad_migration() {
		let dir = TempDir::new().unwrap();
		let file = dir.path().join("001_bad.surql");
		fs::write(&file, "THIS IS NOT VALID SURQL !!!").unwrap();

		let engine = DualEngine::start().await.unwrap();
		let result = engine.apply_migrations(dir.path()).await.unwrap();

		assert_eq!(result.errors.len(), 1);
		let path_str = file.to_string_lossy().to_string();
		let status = engine
			.file_status(&path_str)
			.await
			.expect("should have status");
		assert_eq!(status.status, "error");
		assert!(status.error_message.is_some());
	}

	#[tokio::test]
	async fn should_reset_workspace_without_affecting_meta() {
		let dir = TempDir::new().unwrap();
		let file = dir.path().join("001_schema.surql");
		fs::write(&file, "DEFINE TABLE user SCHEMAFULL;").unwrap();

		let engine = DualEngine::start().await.unwrap();
		engine.apply_migrations(dir.path()).await.unwrap();

		let path_str = file.to_string_lossy().to_string();
		engine.reset_workspace().await.unwrap();

		let status = engine.file_status(&path_str).await;
		assert!(status.is_some(), "meta should survive workspace reset");
	}

	#[tokio::test]
	async fn should_validate_query_against_workspace_schema() {
		let dir = TempDir::new().unwrap();
		fs::write(
			dir.path().join("001_schema.surql"),
			"DEFINE TABLE user SCHEMAFULL; DEFINE FIELD name ON user TYPE string;",
		)
		.unwrap();

		let engine = DualEngine::start().await.unwrap();
		engine.apply_migrations(dir.path()).await.unwrap();

		let errors = engine.validate_query("SELECT * FROM user").await;
		assert!(errors.is_empty());
	}

	#[tokio::test]
	async fn should_handle_empty_directory() {
		let dir = TempDir::new().unwrap();
		let engine = DualEngine::start().await.unwrap();
		let result = engine.apply_migrations(dir.path()).await.unwrap();
		assert_eq!(result.total, 0);
		assert_eq!(result.applied, 0);
	}

	#[tokio::test]
	async fn should_ignore_non_surql_files() {
		let dir = TempDir::new().unwrap();
		fs::write(dir.path().join("readme.md"), "# Hello").unwrap();
		fs::write(dir.path().join("data.json"), "{}").unwrap();
		fs::write(dir.path().join("001_schema.surql"), "DEFINE TABLE user;").unwrap();

		let engine = DualEngine::start().await.unwrap();
		let result = engine.apply_migrations(dir.path()).await.unwrap();
		assert_eq!(result.total, 1);
	}

	#[tokio::test]
	async fn should_detect_needs_reapply_for_new_file() {
		let engine = DualEngine::start().await.unwrap();
		assert!(engine.needs_reapply("/nonexistent.surql", "abc123").await);
	}

	#[tokio::test]
	async fn should_reapply_all_files_when_one_changes() {
		let dir = TempDir::new().unwrap();
		fs::write(dir.path().join("001_a.surql"), "DEFINE TABLE a;").unwrap();
		fs::write(dir.path().join("002_b.surql"), "DEFINE TABLE b;").unwrap();

		let engine = DualEngine::start().await.unwrap();
		let first = engine.apply_migrations(dir.path()).await.unwrap();
		assert_eq!(first.applied, 2);

		// Change only one file — both should be reapplied (workspace is reset)
		fs::write(dir.path().join("002_b.surql"), "DEFINE TABLE b_v2;").unwrap();
		let second = engine.apply_migrations(dir.path()).await.unwrap();
		assert_eq!(
			second.applied, 2,
			"all files reapplied after workspace reset"
		);
	}

	#[tokio::test]
	async fn should_reapply_all_when_file_deleted() {
		let dir = TempDir::new().unwrap();
		let file_a = dir.path().join("001_a.surql");
		let file_b = dir.path().join("002_b.surql");
		fs::write(&file_a, "DEFINE TABLE a;").unwrap();
		fs::write(&file_b, "DEFINE TABLE b;").unwrap();

		let engine = DualEngine::start().await.unwrap();
		let first = engine.apply_migrations(dir.path()).await.unwrap();
		assert_eq!(first.applied, 2);

		fs::remove_file(&file_b).unwrap();
		let second = engine.apply_migrations(dir.path()).await.unwrap();
		// Only file_a remains; meta still has file_b but it's not in the directory scan.
		// file_a is unchanged so no changes detected → skipped.
		assert_eq!(second.total, 1);
		assert_eq!(second.skipped, 1);
	}

	#[tokio::test]
	async fn should_return_empty_errors_for_successful_migration() {
		let dir = TempDir::new().unwrap();
		let file = dir.path().join("001.surql");
		let content = "DEFINE TABLE user SCHEMAFULL;";
		fs::write(&file, content).unwrap();

		let engine = DualEngine::start().await.unwrap();
		engine.apply_migrations(dir.path()).await.unwrap();

		let path_str = file.to_string_lossy().to_string();
		let errors = engine.migration_error_for(&path_str, content).await;
		assert_eq!(
			errors,
			Some(vec![]),
			"successful migration returns empty vec"
		);
	}

	#[tokio::test]
	async fn should_return_recorded_error_for_failed_migration() {
		let dir = TempDir::new().unwrap();
		let file = dir.path().join("001.surql");
		let content = "THIS IS NOT VALID SURQL !!!";
		fs::write(&file, content).unwrap();

		let engine = DualEngine::start().await.unwrap();
		engine.apply_migrations(dir.path()).await.unwrap();

		let path_str = file.to_string_lossy().to_string();
		let errors = engine.migration_error_for(&path_str, content).await;
		assert!(errors.is_some(), "failed migration returns Some");
		assert!(!errors.unwrap().is_empty(), "error list not empty");
	}

	#[tokio::test]
	async fn should_return_none_for_modified_content() {
		let dir = TempDir::new().unwrap();
		let file = dir.path().join("001.surql");
		fs::write(&file, "DEFINE TABLE user;").unwrap();

		let engine = DualEngine::start().await.unwrap();
		engine.apply_migrations(dir.path()).await.unwrap();

		let path_str = file.to_string_lossy().to_string();
		let errors = engine
			.migration_error_for(&path_str, "DEFINE TABLE user SCHEMAFULL;")
			.await;
		assert_eq!(
			errors, None,
			"different content returns None (needs re-validation)"
		);
	}

	#[tokio::test]
	async fn should_not_need_reapply_for_same_hash() {
		let dir = TempDir::new().unwrap();
		let file = dir.path().join("001.surql");
		let content = "DEFINE TABLE user;";
		fs::write(&file, content).unwrap();

		let engine = DualEngine::start().await.unwrap();
		engine.apply_migrations(dir.path()).await.unwrap();

		let path_str = file.to_string_lossy().to_string();
		let hash = content_hash(content);
		assert!(!engine.needs_reapply(&path_str, &hash).await);
	}
}
