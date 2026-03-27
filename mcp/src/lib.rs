use rmcp::{
	handler::server::wrapper::Parameters,
	model::{CallToolResult, Content, Implementation, ServerCapabilities, ServerInfo},
	schemars, tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use surrealdb::{Surreal, engine::local::Mem};
use tokio::sync::RwLock;

pub(crate) mod file_ops;
pub(crate) mod validation;

use file_ops::categorize_files;
pub use file_ops::inject_overwrite;
#[cfg(test)]
pub(crate) use file_ops::{FileCategory, classify_file};
pub(crate) use validation::error_result;
use validation::{is_valid_surql_identifier, validate_path_against};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ExecArgs {
	#[schemars(description = "SurrealQL query to run")]
	pub query: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LoadProjectArgs {
	#[schemars(description = "Path to directory containing .surql files")]
	pub path: String,
	#[schemars(description = "Reset database before loading (default: true)")]
	pub clean: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LoadFileArgs {
	#[schemars(description = "Path to a single .surql file to run")]
	pub path: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DescribeArgs {
	#[schemars(description = "Table name to describe")]
	pub table: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ManifestArgs {
	#[schemars(description = "Path to directory containing manifest.toml (overshift project)")]
	pub path: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CompareArgs {
	#[schemars(
		description = "JSON string from INFO FOR DB on the target database (expected state)"
	)]
	pub expected_json: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct VerifyArgs {
	#[schemars(
		description = "Path to overshift project directory containing manifest.toml. \
			Loads the manifest, applies schema+migrations to both the playground and a \
			fresh shadow DB, then compares INFO FOR DB from both to detect drift."
	)]
	pub path: String,
	#[schemars(
		description = "Read-only mode: only build shadow DB and compare with playground, \
			do NOT apply anything to the playground. Use to safely verify without writes."
	)]
	pub verify_only: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RollbackArgs {
	#[schemars(description = "Path to directory containing manifest.toml (overshift project)")]
	pub path: String,
	#[schemars(
		description = "Target version to roll back to (migrations above this version are reversed)"
	)]
	pub target_version: u32,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CheckArgs {
	#[schemars(description = "Path to a .surql file or directory containing .surql files")]
	pub path: String,
	#[schemars(description = "Recurse into subdirectories (default: true)")]
	pub recursive: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GraphAffectedArgs {
	#[schemars(description = "Table name to check for reverse dependencies")]
	pub table: String,
	#[schemars(
		description = "Path to directory containing .surql files (required for static analysis)"
	)]
	pub schema_path: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GraphTraverseArgs {
	#[schemars(description = "Starting table name")]
	pub table: String,
	#[schemars(
		description = "Path to directory containing .surql files (required for static analysis)"
	)]
	pub schema_path: String,
	#[schemars(description = "Maximum traversal depth (default: 10)")]
	pub depth: Option<u32>,
	#[schemars(description = "Traversal direction: 'forward' (default) or 'reverse'")]
	pub direction: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GraphSiblingsArgs {
	#[schemars(description = "Table name to find siblings for")]
	pub table: String,
	#[schemars(
		description = "Path to directory containing .surql files (required for static analysis)"
	)]
	pub schema_path: String,
}

#[derive(Clone)]
pub struct SurqlMcp {
	db: Arc<RwLock<Surreal<surrealdb::engine::local::Db>>>,
	query_count: Arc<AtomicU64>,
	workspace_root: Arc<PathBuf>,
	tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
}

const QUERY_WARNING_THRESHOLDS: &[u64] = &[1000, 5000, 10000];

#[tool_router]
impl SurqlMcp {
	pub async fn new() -> anyhow::Result<Self> {
		let cwd = std::env::current_dir()?;
		Self::with_workspace_root(cwd).await
	}

	pub async fn with_workspace_root(root: PathBuf) -> anyhow::Result<Self> {
		let db = Surreal::new::<Mem>(()).await?;
		db.use_ns("default").use_db("default").await?;
		tracing::info!("SurrealDB playground started (root: {})", root.display());
		Ok(Self {
			db: Arc::new(RwLock::new(db)),
			query_count: Arc::new(AtomicU64::new(0)),
			workspace_root: Arc::new(root),
			tool_router: Self::tool_router(),
		})
	}

	fn increment_query_count(&self) {
		let count = self.query_count.fetch_add(1, Ordering::Relaxed) + 1;
		if QUERY_WARNING_THRESHOLDS.contains(&count) {
			tracing::warn!(
				"In-memory DB has processed {count} queries — \
				 consider resetting with the 'reset' tool if performance degrades"
			);
		}
	}

	#[tool(
		name = "exec",
		description = "Run a SurrealQL query and return the result as JSON"
	)]
	pub async fn run_query(
		&self,
		Parameters(args): Parameters<ExecArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
		self.increment_query_count();
		let db = self.db.read().await;
		match db.query(&args.query).await {
			Ok(response) => match response.check() {
				Ok(mut checked) => {
					let result: Result<Vec<serde_json::Value>, _> = checked.take(0);
					match result {
						Ok(rows) => {
							let json = serde_json::to_string_pretty(&rows)
								.unwrap_or_else(|_| "[]".to_string());
							let summary = if rows.is_empty() {
								"(empty result)".to_string()
							} else {
								format!(
									"{} row{}",
									rows.len(),
									if rows.len() == 1 { "" } else { "s" }
								)
							};
							Ok(CallToolResult::success(vec![Content::text(format!(
								"{summary}\n\n```json\n{json}\n```"
							))]))
						}
						Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
							"Query ran but result extraction failed: {e}"
						))])),
					}
				}
				Err(e) => error_result(format!("Query error: {e}")),
			},
			Err(e) => error_result(format!("Query failed: {e}")),
		}
	}

	#[tool(
		name = "load_project",
		description = "Load .surql files from a directory into the database. Resets DB first \
			by default. Files are categorized by directory: schema/ files get OVERWRITE \
			injected, migrations/ run in version order, examples/ errors are warnings."
	)]
	pub async fn load_project(
		&self,
		Parameters(args): Parameters<LoadProjectArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
		let dir = match validate_path_against(&args.path, &self.workspace_root) {
			Ok(p) => p,
			Err(e) => return error_result(e),
		};
		if !dir.is_dir() {
			return error_result(format!("Not a directory: {}", args.path));
		}

		let clean = args.clean.unwrap_or(true);
		if clean {
			let db = self.db.read().await;
			// Reset to known state: switch to default NS/DB first, then remove the database
			if let Err(e) = db.use_ns("default").use_db("default").await {
				return error_result(format!("Failed to reset namespace: {e}"));
			}
			db.query("REMOVE DATABASE IF EXISTS default").await.ok();
			// Re-create the default database after removal
			if let Err(e) = db.use_ns("default").use_db("default").await {
				return error_result(format!("Failed to re-create default database: {e}"));
			}
		}

		let mut surql_files = Vec::new();
		surql_parser::collect_surql_files(&dir, &mut surql_files);

		if surql_files.is_empty() {
			return Ok(CallToolResult::success(vec![Content::text(
				"No .surql files found",
			)]));
		}

		let categorized = categorize_files(&surql_files);

		let db = self.db.read().await;
		let mut schema_count = 0usize;
		let mut migration_count = 0usize;
		let mut function_count = 0usize;
		let mut example_count = 0usize;
		let mut errors = Vec::new();
		let mut warnings = Vec::new();

		// 1. Schema files (with OVERWRITE injection)
		for path in &categorized.schema {
			let content = match surql_parser::read_surql_file(path) {
				Ok(c) => inject_overwrite(&c),
				Err(e) => {
					errors.push(e);
					continue;
				}
			};
			match db.query(&content).await {
				Ok(response) => match response.check() {
					Ok(_) => schema_count += 1,
					Err(e) => errors.push(format!("{}: {e}", path.display())),
				},
				Err(e) => errors.push(format!("{}: {e}", path.display())),
			}
		}

		// 2. Function files (with OVERWRITE injection)
		for path in &categorized.functions {
			let content = match surql_parser::read_surql_file(path) {
				Ok(c) => inject_overwrite(&c),
				Err(e) => {
					errors.push(e);
					continue;
				}
			};
			match db.query(&content).await {
				Ok(response) => match response.check() {
					Ok(_) => function_count += 1,
					Err(e) => errors.push(format!("{}: {e}", path.display())),
				},
				Err(e) => errors.push(format!("{}: {e}", path.display())),
			}
		}

		// 3. Migration files (in version order, one-shot)
		let mut migrations = categorized.migrations.clone();
		migrations.sort();
		for path in &migrations {
			let content = match surql_parser::read_surql_file(path) {
				Ok(c) => c,
				Err(e) => {
					errors.push(e);
					continue;
				}
			};
			match db.query(&content).await {
				Ok(response) => match response.check() {
					Ok(_) => migration_count += 1,
					Err(e) => errors.push(format!("{}: {e}", path.display())),
				},
				Err(e) => errors.push(format!("{}: {e}", path.display())),
			}
		}

		// 4. General files
		for path in &categorized.general {
			let content = match surql_parser::read_surql_file(path) {
				Ok(c) => c,
				Err(e) => {
					errors.push(e);
					continue;
				}
			};
			match db.query(&content).await {
				Ok(response) => match response.check() {
					Ok(_) => {}
					Err(e) => errors.push(format!("{}: {e}", path.display())),
				},
				Err(e) => errors.push(format!("{}: {e}", path.display())),
			}
		}

		// 5. Example files (errors become warnings)
		for path in &categorized.examples {
			let content = match surql_parser::read_surql_file(path) {
				Ok(c) => c,
				Err(e) => {
					warnings.push(e);
					continue;
				}
			};
			match db.query(&content).await {
				Ok(response) => match response.check() {
					Ok(_) => example_count += 1,
					Err(e) => {
						warnings.push(format!("{}: {e}", path.display()));
					}
				},
				Err(e) => {
					warnings.push(format!("{}: {e}", path.display()));
				}
			}
		}

		let mut output = format!(
			"Loaded {} schema, {} migrations, {} functions, {} examples ({} warnings) from `{}`{}",
			schema_count,
			migration_count,
			function_count,
			example_count,
			warnings.len(),
			args.path,
			if clean { " (clean)" } else { "" }
		);
		if !errors.is_empty() {
			output.push_str(&format!(
				"\n\n**Errors ({}):**\n{}",
				errors.len(),
				errors.join("\n")
			));
		}
		if !warnings.is_empty() {
			output.push_str(&format!(
				"\n\n**Warnings ({}):**\n{}",
				warnings.len(),
				warnings.join("\n")
			));
		}
		Ok(CallToolResult::success(vec![Content::text(output)]))
	}

	#[tool(
		name = "load_file",
		description = "Run a single .surql file against the database"
	)]
	pub async fn load_file(
		&self,
		Parameters(args): Parameters<LoadFileArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
		self.increment_query_count();
		let path = match validate_path_against(&args.path, &self.workspace_root) {
			Ok(p) => p,
			Err(e) => return error_result(e),
		};
		let content = match surql_parser::read_surql_file(&path) {
			Ok(c) => c,
			Err(e) => return error_result(e),
		};
		let db = self.db.read().await;
		match db.query(&content).await {
			Ok(response) => match response.check() {
				Ok(_) => Ok(CallToolResult::success(vec![Content::text(format!(
					"Applied `{}`",
					path.file_name()
						.and_then(|n| n.to_str())
						.unwrap_or(&args.path)
				))])),
				Err(e) => error_result(format!("{e}")),
			},
			Err(e) => error_result(format!("{e}")),
		}
	}

	#[tool(
		name = "schema",
		description = "Show all tables, fields, indexes, and events in the current database"
	)]
	pub async fn schema(&self) -> Result<CallToolResult, rmcp::ErrorData> {
		let db = self.db.read().await;
		let mut response = match db.query("INFO FOR DB").await {
			Ok(r) => r,
			Err(e) => return error_result(format!("Failed: {e}")),
		};
		let info: Result<Option<serde_json::Value>, _> = response.take(0);
		match info {
			Ok(Some(val)) => {
				let json = serde_json::to_string_pretty(&val).unwrap_or_else(|_| "{}".to_string());
				Ok(CallToolResult::success(vec![Content::text(format!(
					"```json\n{json}\n```"
				))]))
			}
			Ok(None) => Ok(CallToolResult::success(vec![Content::text(
				"(empty database)",
			)])),
			Err(e) => error_result(format!("Failed to read schema: {e}")),
		}
	}

	#[tool(
		name = "describe",
		description = "Show detailed info about a specific table (fields, indexes, events)"
	)]
	pub async fn describe(
		&self,
		Parameters(args): Parameters<DescribeArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
		if args.table.contains('`') {
			return error_result("Table name must not contain backticks".into());
		}
		if !is_valid_surql_identifier(&args.table) {
			return error_result(format!("Invalid table name: {}", args.table));
		}
		let db = self.db.read().await;
		let query = format!("INFO FOR TABLE `{}`", args.table);
		let mut response = match db.query(&query).await {
			Ok(r) => r,
			Err(e) => return error_result(format!("Failed: {e}")),
		};
		let info: Result<Option<serde_json::Value>, _> = response.take(0);
		match info {
			Ok(Some(val)) => {
				let json = serde_json::to_string_pretty(&val).unwrap_or_else(|_| "{}".to_string());
				Ok(CallToolResult::success(vec![Content::text(format!(
					"**Table `{}`**\n\n```json\n{json}\n```",
					args.table
				))]))
			}
			Ok(None) => error_result(format!("Table '{}' not found", args.table)),
			Err(e) => error_result(format!("Failed: {e}")),
		}
	}

	#[tool(
		name = "manifest",
		description = "Read an overshift manifest.toml and show project configuration \
			(namespace, database, modules, migrations)"
	)]
	pub async fn manifest(
		&self,
		Parameters(args): Parameters<ManifestArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
		let validated = match validate_path_against(&args.path, &self.workspace_root) {
			Ok(p) => p,
			Err(e) => return error_result(e),
		};
		let manifest = match overshift::Manifest::load(&validated) {
			Ok(m) => m,
			Err(e) => return error_result(format!("Cannot load manifest: {e}")),
		};

		let mut output = format!(
			"**overshift manifest** from `{}`\n\n\
			 - **Namespace:** `{}`\n\
			 - **Database:** `{}`\n\
			 - **System DB:** `{}`\n",
			args.path, manifest.meta.ns, manifest.meta.db, manifest.meta.system_db
		);
		if let Some(ver) = &manifest.meta.surrealdb {
			output.push_str(&format!("- **SurrealDB:** `{ver}`\n"));
		}

		if !manifest.modules.is_empty() {
			output.push_str(&format!("\n**{} module(s):**\n", manifest.modules.len()));
			for m in &manifest.modules {
				let deps = if m.depends_on.is_empty() {
					String::new()
				} else {
					format!(" (depends: {})", m.depends_on.join(", "))
				};
				output.push_str(&format!("- `{}` \u{2192} `{}`{deps}\n", m.name, m.path));
			}
		}

		// Discover migrations via overshift
		let migrations = overshift::migration::discover_migrations(
			manifest.root_path().unwrap_or(std::path::Path::new(".")),
		);
		match migrations {
			Ok(migs) if !migs.is_empty() => {
				output.push_str(&format!("\n**{} migration(s):**\n", migs.len()));
				for m in &migs {
					output.push_str(&format!("- `{}` ({})\n", m.name, &m.checksum[..8]));
				}
			}
			_ => {}
		}

		Ok(CallToolResult::success(vec![Content::text(output)]))
	}

	#[tool(
		name = "load_manifest",
		description = "Load an overshift project into the playground DB: applies schema \
			modules then migrations in order"
	)]
	pub async fn load_manifest(
		&self,
		Parameters(args): Parameters<ManifestArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
		let validated = match validate_path_against(&args.path, &self.workspace_root) {
			Ok(p) => p,
			Err(e) => return error_result(e),
		};
		let manifest = match overshift::Manifest::load(&validated) {
			Ok(m) => m,
			Err(e) => return error_result(format!("Cannot load manifest: {e}")),
		};

		// Reset DB
		let db = self.db.read().await;
		// REMOVE DATABASE may fail if it doesn't exist yet -- safe to ignore
		db.query("REMOVE DATABASE default").await.ok();
		if let Err(e) = db.use_ns(&manifest.meta.ns).use_db(&manifest.meta.db).await {
			return error_result(format!(
				"Failed to switch to NS={}/DB={}: {e}",
				manifest.meta.ns, manifest.meta.db
			));
		}

		let mut applied = 0;
		let mut errors = Vec::new();

		// Apply schema modules first
		let modules = match overshift::schema::load_schema_modules(&manifest) {
			Ok(m) => m,
			Err(e) => return error_result(format!("Failed to load schema modules: {e}")),
		};
		for module in &modules {
			match db.query(&module.content).await {
				Ok(r) => match r.check() {
					Ok(_) => applied += 1,
					Err(e) => errors.push(format!("schema/{}: {e}", module.name)),
				},
				Err(e) => errors.push(format!("schema/{}: {e}", module.name)),
			}
		}

		// Then migrations
		let migrations = match overshift::migration::discover_migrations(
			manifest.root_path().unwrap_or(std::path::Path::new(".")),
		) {
			Ok(m) => m,
			Err(e) => return error_result(format!("Failed to discover migrations: {e}")),
		};
		for mig in &migrations {
			match db.query(mig.content.as_str()).await {
				Ok(r) => match r.check() {
					Ok(_) => applied += 1,
					Err(e) => errors.push(format!("{}: {e}", mig.name)),
				},
				Err(e) => errors.push(format!("{}: {e}", mig.name)),
			}
		}

		let mut output = format!(
			"Loaded overshift project `{}` (NS={}, DB={})\n\
			 {} schema module(s) + {} migration(s) = {applied} applied",
			args.path,
			manifest.meta.ns,
			manifest.meta.db,
			modules.len(),
			migrations.len()
		);
		if !errors.is_empty() {
			output.push_str(&format!(
				"\n\n**Errors ({}):**\n{}",
				errors.len(),
				errors.join("\n")
			));
		}
		Ok(CallToolResult::success(vec![Content::text(output)]))
	}

	#[tool(
		name = "compare",
		description = "Compare the playground DB schema against an expected INFO FOR DB \
			JSON response. Returns a diff of missing/extra tables and functions."
	)]
	pub async fn compare(
		&self,
		Parameters(args): Parameters<CompareArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
		let expected: serde_json::Value = match serde_json::from_str(&args.expected_json) {
			Ok(v) => v,
			Err(e) => return error_result(format!("Invalid expected_json: {e}")),
		};

		let db = self.db.read().await;
		let mut response = match db.query("INFO FOR DB").await {
			Ok(r) => r,
			Err(e) => return error_result(format!("Failed to query playground: {e}")),
		};
		let actual: Option<serde_json::Value> = match response.take(0) {
			Ok(v) => v,
			Err(e) => return error_result(format!("Failed to read playground schema: {e}")),
		};
		let actual = match actual {
			Some(v) => v,
			None => return error_result("INFO FOR DB returned no data".into()),
		};

		let diff = overshift::validate::compare_db_info(&expected, &actual);
		let output = diff.to_string();
		Ok(CallToolResult::success(vec![Content::text(output)]))
	}

	#[tool(
		name = "verify",
		description = "Verify an overshift project by applying it to both the playground \
			DB and a fresh shadow in-memory DB, then comparing their schemas via INFO FOR \
			DB. Detects drift between the two environments."
	)]
	pub async fn verify(
		&self,
		Parameters(args): Parameters<VerifyArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
		let validated = match validate_path_against(&args.path, &self.workspace_root) {
			Ok(p) => p,
			Err(e) => return error_result(e),
		};
		let manifest = match overshift::Manifest::load(&validated) {
			Ok(m) => m,
			Err(e) => return error_result(format!("Cannot load manifest: {e}")),
		};

		let modules = match overshift::schema::load_schema_modules(&manifest) {
			Ok(m) => m,
			Err(e) => return error_result(format!("Failed to load schema modules: {e}")),
		};

		let migrations = match overshift::migration::discover_migrations(
			manifest.root_path().unwrap_or(std::path::Path::new(".")),
		) {
			Ok(m) => m,
			Err(e) => return error_result(format!("Failed to discover migrations: {e}")),
		};

		let verify_only = args.verify_only.unwrap_or(false);

		// Apply to playground DB (skip in verify-only mode)
		if !verify_only {
			if !is_valid_surql_identifier(&manifest.meta.ns)
				|| !is_valid_surql_identifier(&manifest.meta.db)
			{
				return error_result(format!(
					"Invalid NS/DB in manifest: NS={}, DB={}",
					manifest.meta.ns, manifest.meta.db
				));
			}
			let db = self.db.read().await;
			if let Err(e) = db.use_ns(&manifest.meta.ns).use_db("default").await {
				return error_result(format!("Failed to switch to NS={}: {e}", manifest.meta.ns));
			}
			db.query(format!(
				"REMOVE DATABASE IF EXISTS `{}`",
				manifest.meta.db.replace('`', "")
			))
			.await
			.ok();
			if let Err(e) = db.use_ns(&manifest.meta.ns).use_db(&manifest.meta.db).await {
				return error_result(format!(
					"Failed to switch playground to NS={}/DB={}: {e}",
					manifest.meta.ns, manifest.meta.db
				));
			}

			for module in &modules {
				if let Err(e) = db.query(&module.content).await.and_then(|r| r.check()) {
					return error_result(format!(
						"Playground: schema module '{}' failed: {e}",
						module.name
					));
				}
			}
			for mig in &migrations {
				if let Err(e) = db.query(mig.content.as_str()).await.and_then(|r| r.check()) {
					return error_result(format!(
						"Playground: migration '{}' failed: {e}",
						mig.name
					));
				}
			}
		}

		// Create shadow in-memory DB and apply the same project
		let shadow_db = match Surreal::new::<Mem>(()).await {
			Ok(db) => db,
			Err(e) => return error_result(format!("Failed to create shadow DB: {e}")),
		};
		if let Err(e) = shadow_db
			.use_ns(&manifest.meta.ns)
			.use_db(&manifest.meta.db)
			.await
		{
			return error_result(format!(
				"Failed to switch shadow to NS={}/DB={}: {e}",
				manifest.meta.ns, manifest.meta.db
			));
		}

		for module in &modules {
			if let Err(e) = shadow_db
				.query(&module.content)
				.await
				.and_then(|r| r.check())
			{
				return error_result(format!(
					"Shadow: schema module '{}' failed: {e}",
					module.name
				));
			}
		}
		for mig in &migrations {
			if let Err(e) = shadow_db
				.query(mig.content.as_str())
				.await
				.and_then(|r| r.check())
			{
				return error_result(format!("Shadow: migration '{}' failed: {e}", mig.name));
			}
		}

		if verify_only {
			let shadow_info = {
				let mut resp = match shadow_db.query("INFO FOR DB").await {
					Ok(r) => r,
					Err(e) => {
						return error_result(format!("Failed to query shadow INFO FOR DB: {e}"));
					}
				};
				let val: Option<serde_json::Value> = match resp.take(0) {
					Ok(v) => v,
					Err(e) => {
						return error_result(format!("Failed to read shadow schema: {e}"));
					}
				};
				match val {
					Some(v) => v,
					None => return error_result("Shadow INFO FOR DB returned no data".into()),
				}
			};
			let shadow_text =
				serde_json::to_string_pretty(&shadow_info).unwrap_or_else(|_| "{}".to_string());
			return Ok(CallToolResult::success(vec![Content::text(format!(
				"Shadow verification (read-only)\n\
				 {} module(s), {} migration(s)\n\n\
				 ```json\n{shadow_text}\n```",
				modules.len(),
				migrations.len(),
			))]));
		}

		// Get INFO FOR DB from both
		let playground_info = {
			let db = self.db.read().await;
			let mut resp = match db.query("INFO FOR DB").await {
				Ok(r) => r,
				Err(e) => {
					return error_result(format!("Failed to query playground INFO FOR DB: {e}"));
				}
			};
			let val: Option<serde_json::Value> = match resp.take(0) {
				Ok(v) => v,
				Err(e) => return error_result(format!("Failed to read playground schema: {e}")),
			};
			match val {
				Some(v) => v,
				None => return error_result("Playground INFO FOR DB returned no data".into()),
			}
		};

		let shadow_info = {
			let mut resp = match shadow_db.query("INFO FOR DB").await {
				Ok(r) => r,
				Err(e) => return error_result(format!("Failed to query shadow INFO FOR DB: {e}")),
			};
			let val: Option<serde_json::Value> = match resp.take(0) {
				Ok(v) => v,
				Err(e) => return error_result(format!("Failed to read shadow schema: {e}")),
			};
			match val {
				Some(v) => v,
				None => return error_result("Shadow INFO FOR DB returned no data".into()),
			}
		};

		let diff = overshift::validate::compare_db_info(&playground_info, &shadow_info);

		let mut output = format!(
			"**Verify** `{}` (NS={}, DB={})\n\
			 Applied {} module(s) + {} migration(s) to playground\n\
			 Applied {} module(s) + {} migration(s) to shadow\n\n",
			args.path,
			manifest.meta.ns,
			manifest.meta.db,
			modules.len(),
			migrations.len(),
			modules.len(),
			migrations.len(),
		);

		if diff.is_empty() {
			output.push_str("Schema matches -- playground and shadow are identical.");
		} else {
			output.push_str(&format!("**Drift detected:**\n{diff}"));
		}

		Ok(CallToolResult::success(vec![Content::text(output)]))
	}

	#[tool(
		name = "check",
		description = "Parse .surql files and report syntax errors without executing. \
            Path can be a single file or a directory."
	)]
	pub async fn check(
		&self,
		Parameters(args): Parameters<CheckArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
		let path = match validate_path_against(&args.path, &self.workspace_root) {
			Ok(p) => p,
			Err(e) => return error_result(e),
		};
		let recursive = args.recursive.unwrap_or(true);

		let files: Vec<PathBuf> = if path.is_file() {
			vec![path]
		} else if path.is_dir() {
			if recursive {
				let mut collected = Vec::new();
				surql_parser::collect_surql_files(&path, &mut collected);
				collected
			} else {
				match std::fs::read_dir(&path) {
					Ok(entries) => entries
						.filter_map(|e| e.ok())
						.map(|e| e.path())
						.filter(|p| {
							p.extension()
								.and_then(|ext| ext.to_str())
								.is_some_and(|ext| ext == "surql")
						})
						.collect(),
					Err(e) => {
						return error_result(format!("Cannot read directory {}: {e}", args.path));
					}
				}
			}
		} else {
			return error_result(format!("Path does not exist: {}", args.path));
		};

		if files.is_empty() {
			return Ok(CallToolResult::success(vec![Content::text(
				"No .surql files found",
			)]));
		}

		let mut seen = std::collections::HashSet::new();
		let mut total_errors = 0usize;
		let mut error_details = Vec::new();

		for file in &files {
			let canonical = file.canonicalize().unwrap_or_else(|_| file.clone());
			if !seen.insert(canonical.clone()) {
				continue;
			}
			let content = match surql_parser::read_surql_file(&canonical) {
				Ok(c) => c,
				Err(e) => {
					error_details.push(format!("{}:0:0: {e}", file.display()));
					total_errors += 1;
					continue;
				}
			};
			if let Err(diags) = surql_parser::parse_for_diagnostics(&content) {
				for d in &diags {
					error_details.push(format!(
						"{}:{}:{}: {}",
						file.display(),
						d.line,
						d.column,
						d.message
					));
				}
				total_errors += diags.len();
			}
		}

		let file_count = seen.len();
		let mut output = format!(
			"{file_count} file{} checked, {total_errors} error{} found",
			if file_count == 1 { "" } else { "s" },
			if total_errors == 1 { "" } else { "s" },
		);
		if !error_details.is_empty() {
			output.push_str("\n\n");
			output.push_str(&error_details.join("\n"));
		}

		Ok(CallToolResult::success(vec![Content::text(output)]))
	}

	#[tool(
		name = "rollback",
		description = "Roll back applied migrations in an overshift project to a target version. \
			Resets the playground and re-applies schema modules + migrations up to target_version. \
			In-memory playground rebuilds from scratch (no down.surql needed)."
	)]
	pub async fn rollback(
		&self,
		Parameters(args): Parameters<RollbackArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
		let validated = match validate_path_against(&args.path, &self.workspace_root) {
			Ok(p) => p,
			Err(e) => return error_result(e),
		};
		let manifest = match overshift::Manifest::load(&validated) {
			Ok(m) => m,
			Err(e) => return error_result(format!("Cannot load manifest: {e}")),
		};

		let db = self.db.read().await;

		// Reset playground: switch to NS/DB then remove+recreate so we rebuild from scratch.
		// In-memory playground does not need down.surql — we rebuild state up to target_version.
		let remove_sql = format!("REMOVE DATABASE IF EXISTS {}", manifest.meta.db);
		db.query(&remove_sql).await.ok();
		if let Err(e) = db.use_ns(&manifest.meta.ns).use_db(&manifest.meta.db).await {
			return error_result(format!(
				"Failed to switch to NS={}/DB={}: {e}",
				manifest.meta.ns, manifest.meta.db
			));
		}

		// Re-apply schema modules (same as load_manifest)
		let modules = match overshift::schema::load_schema_modules(&manifest) {
			Ok(m) => m,
			Err(e) => return error_result(format!("Failed to load schema modules: {e}")),
		};
		let mut schema_applied = 0u32;
		for module in &modules {
			let injected = inject_overwrite(&module.content);
			if let Err(e) = db.query(&injected).await.and_then(|r| r.check()) {
				return error_result(format!("Schema module {} failed: {e}", module.name));
			}
			schema_applied += 1;
		}

		// Discover and re-apply migrations only up to target_version
		let target = args.target_version;
		let migrations = match overshift::migration::discover_migrations(
			manifest.root_path().unwrap_or(std::path::Path::new(".")),
		) {
			Ok(m) => m,
			Err(e) => return error_result(format!("Failed to discover migrations: {e}")),
		};
		let total_migrations = migrations.len() as u32;
		let mut migrations_applied = 0u32;
		let mut total_rolled_back = 0u32;
		for mig in &migrations {
			if mig.version > target {
				total_rolled_back += 1;
				continue;
			}
			if let Err(e) = db.query(mig.content.as_str()).await.and_then(|r| r.check()) {
				return error_result(format!("Migration {} failed: {e}", mig.name));
			}
			migrations_applied += 1;
		}

		let mut output = format!(
			"**Rollback** `{}` to v{target:03}\n\
			 {schema_applied} schema module(s) re-applied, \
			 {migrations_applied} migration(s) re-applied, \
			 {total_rolled_back} migration(s) rolled back",
			args.path,
		);

		if total_migrations > 0 {
			let max_version = migrations.last().map(|m| m.version).unwrap_or(0);
			output.push_str(&format!("\n\nState: v{target:03} of v{max_version:03}"));
		}

		Ok(CallToolResult::success(vec![Content::text(output)]))
	}

	#[tool(
		name = "graph_affected",
		description = "Show which tables would be affected if a table is dropped or modified. \
			Follows record<> links in reverse to find all dependents."
	)]
	pub async fn graph_affected(
		&self,
		Parameters(args): Parameters<GraphAffectedArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
		let dir = match validate_path_against(&args.schema_path, &self.workspace_root) {
			Ok(p) => p,
			Err(e) => return error_result(e),
		};
		if !dir.is_dir() {
			return error_result(format!("Not a directory: {}", args.schema_path));
		}
		let graph = match surql_parser::SchemaGraph::from_files(&dir) {
			Ok(g) => g,
			Err(e) => return error_result(format!("Failed to build schema graph: {e}")),
		};
		if graph.table(&args.table).is_none() {
			return error_result(format!(
				"Table '{}' not found in schema at {}",
				args.table, args.schema_path
			));
		}

		let refs = graph.tables_referencing(&args.table);
		if refs.is_empty() {
			return Ok(CallToolResult::success(vec![Content::text(format!(
				"No tables reference `{}`",
				args.table
			))]));
		}

		let mut output = format!(
			"**{} table(s) reference `{}`:**\n\n",
			refs.len(),
			args.table
		);
		for (ref_table, ref_field) in &refs {
			output.push_str(&format!(
				"- `{ref_table}.{ref_field}` has `record<{}>`\n",
				args.table
			));
		}

		output.push_str(&format!(
			"\nDropping or renaming `{}` would break {} field(s).",
			args.table,
			refs.len()
		));

		Ok(CallToolResult::success(vec![Content::text(output)]))
	}

	#[tool(
		name = "graph_traverse",
		description = "Traverse the schema graph from a table, following record<> links \
			up to N hops deep. Supports forward (outgoing links) and reverse (incoming links) \
			directions."
	)]
	pub async fn graph_traverse(
		&self,
		Parameters(args): Parameters<GraphTraverseArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
		let dir = match validate_path_against(&args.schema_path, &self.workspace_root) {
			Ok(p) => p,
			Err(e) => return error_result(e),
		};
		if !dir.is_dir() {
			return error_result(format!("Not a directory: {}", args.schema_path));
		}
		let graph = match surql_parser::SchemaGraph::from_files(&dir) {
			Ok(g) => g,
			Err(e) => return error_result(format!("Failed to build schema graph: {e}")),
		};
		if graph.table(&args.table).is_none() {
			return error_result(format!(
				"Table '{}' not found in schema at {}",
				args.table, args.schema_path
			));
		}

		let depth = args.depth.unwrap_or(10) as usize;
		let direction = args.direction.as_deref().unwrap_or("forward");

		match direction {
			"forward" => {
				let reachable = graph.tables_reachable_from(&args.table, depth);
				if reachable.is_empty() {
					return Ok(CallToolResult::success(vec![Content::text(format!(
						"`{}` has no outgoing record<> links",
						args.table
					))]));
				}

				let tree = graph.dependency_tree(&args.table, depth);
				let mut output = format!(
					"**Forward traversal from `{}`** (max depth: {depth})\n\n\
					 {} table(s) reachable:\n\n```\n",
					args.table,
					reachable.len()
				);
				output.push_str(&format!("[{}]\n", tree.table));
				for child in &tree.children {
					format_dependency_node_mcp(&mut output, child, 1);
				}
				output.push_str("```\n\n**Flat list:**\n");
				for (name, d, path) in &reachable {
					let path_str = path.join(" -> ");
					output.push_str(&format!("- `{name}` (depth {d}): {path_str}\n"));
				}
				Ok(CallToolResult::success(vec![Content::text(output)]))
			}
			"reverse" => {
				let refs = graph.tables_referencing(&args.table);
				if refs.is_empty() {
					return Ok(CallToolResult::success(vec![Content::text(format!(
						"No tables reference `{}`",
						args.table
					))]));
				}

				let mut output = format!(
					"**Reverse traversal for `{}`**\n\n\
					 {} table(s) reference it:\n\n",
					args.table,
					refs.len()
				);
				for (ref_table, ref_field) in &refs {
					output.push_str(&format!(
						"- `{ref_table}.{ref_field}` -> `{}`\n",
						args.table
					));
				}
				Ok(CallToolResult::success(vec![Content::text(output)]))
			}
			other => error_result(format!(
				"Invalid direction '{other}': must be 'forward' or 'reverse'"
			)),
		}
	}

	#[tool(
		name = "graph_siblings",
		description = "Find tables that share record<> link targets with a given table. \
			Shows which other tables also point to the same targets."
	)]
	pub async fn graph_siblings(
		&self,
		Parameters(args): Parameters<GraphSiblingsArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
		let dir = match validate_path_against(&args.schema_path, &self.workspace_root) {
			Ok(p) => p,
			Err(e) => return error_result(e),
		};
		if !dir.is_dir() {
			return error_result(format!("Not a directory: {}", args.schema_path));
		}
		let graph = match surql_parser::SchemaGraph::from_files(&dir) {
			Ok(g) => g,
			Err(e) => return error_result(format!("Failed to build schema graph: {e}")),
		};
		if graph.table(&args.table).is_none() {
			return error_result(format!(
				"Table '{}' not found in schema at {}",
				args.table, args.schema_path
			));
		}

		let siblings = graph.siblings_of(&args.table);
		if siblings.is_empty() {
			return Ok(CallToolResult::success(vec![Content::text(format!(
				"`{}` shares no record<> targets with other tables",
				args.table
			))]));
		}

		let mut output = format!("**Siblings of `{}`:**\n\n", args.table);
		for (sib, target, field) in &siblings {
			output.push_str(&format!(
				"- `{sib}` also links to `{target}` (via `.{field}`)\n"
			));
		}
		Ok(CallToolResult::success(vec![Content::text(output)]))
	}

	#[tool(name = "reset", description = "Clear the database and start fresh")]
	pub async fn reset(&self) -> Result<CallToolResult, rmcp::ErrorData> {
		let db = self.db.read().await;
		// REMOVE DATABASE may fail if it doesn't exist yet -- safe to ignore
		db.query("REMOVE DATABASE default").await.ok();
		if let Err(e) = db.use_ns("default").use_db("default").await {
			return error_result(format!("Reset failed: {e}"));
		}
		Ok(CallToolResult::success(vec![Content::text(
			"Database cleared",
		)]))
	}
}

#[tool_handler]
impl rmcp::handler::server::ServerHandler for SurqlMcp {
	fn get_info(&self) -> ServerInfo {
		ServerInfo {
			instructions: Some(
				"SurrealQL playground: run queries, load schema files, explore database".into(),
			),
			capabilities: ServerCapabilities::builder().enable_tools().build(),
			server_info: Implementation {
				name: "surql-mcp".into(),
				version: env!("CARGO_PKG_VERSION").into(),
				title: None,
				description: None,
				icons: None,
				website_url: None,
			},
			..Default::default()
		}
	}
}

fn format_dependency_node_mcp(
	out: &mut String,
	node: &surql_parser::DependencyNode,
	indent: usize,
) {
	let prefix = "  ".repeat(indent);
	let field_label = node
		.field
		.as_deref()
		.map(|f| format!(".{f} -> "))
		.unwrap_or_default();
	let cycle_label = if node.is_cycle { " (cycle)" } else { "" };
	out.push_str(&format!(
		"{prefix}{field_label}[{}]{cycle_label}\n",
		node.table
	));
	if !node.is_cycle {
		for child in &node.children {
			format_dependency_node_mcp(out, child, indent + 1);
		}
	}
}

pub fn result_text(result: &CallToolResult) -> String {
	result
		.content
		.iter()
		.filter_map(|c| match &c.raw {
			rmcp::model::RawContent::Text(t) => Some(t.text.as_str()),
			_ => None,
		})
		.collect::<Vec<_>>()
		.join("\n")
}

#[cfg(test)]
mod tests {
	use super::*;
	use rmcp::handler::server::wrapper::Parameters;
	use std::fs;
	use tempfile::TempDir;

	#[tokio::test]
	async fn should_start_and_run_query_return() {
		let server = SurqlMcp::new().await.unwrap();
		let result = server
			.run_query(Parameters(ExecArgs {
				query: "RETURN 42".into(),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(text.contains("42"), "expected 42 in: {text}");
	}

	#[tokio::test]
	async fn should_run_query_create_and_select() {
		let server = SurqlMcp::new().await.unwrap();
		server
			.run_query(Parameters(ExecArgs {
				query: "CREATE user:alice SET name = 'Alice'".into(),
			}))
			.await
			.unwrap();

		let result = server
			.run_query(Parameters(ExecArgs {
				query: "SELECT * FROM user".into(),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(text.contains("Alice"), "expected Alice in: {text}");
		assert!(text.contains("1 row"), "expected 1 row in: {text}");
	}

	#[tokio::test]
	async fn should_run_query_select_from_nonexistent_returns_empty() {
		let server = SurqlMcp::new().await.unwrap();
		let result = server
			.run_query(Parameters(ExecArgs {
				query: "SELECT * FROM nonexistent".into(),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(
			text.contains("empty") || result.is_error == Some(true),
			"nonexistent table should return empty or error: {text}"
		);
	}

	#[tokio::test]
	async fn should_run_query_report_syntax_error() {
		let server = SurqlMcp::new().await.unwrap();
		let result = server
			.run_query(Parameters(ExecArgs {
				query: "NOT VALID SQL !!!".into(),
			}))
			.await
			.unwrap();
		assert!(result.is_error == Some(true));
	}

	#[tokio::test]
	async fn should_load_project_from_directory() {
		let dir = TempDir::new().unwrap();
		fs::create_dir_all(dir.path().join("schema")).unwrap();
		fs::write(
			dir.path().join("schema/tables.surql"),
			"DEFINE TABLE user SCHEMAFULL; DEFINE FIELD name ON user TYPE string;",
		)
		.unwrap();

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();
		let result = server
			.load_project(Parameters(LoadProjectArgs {
				path: dir.path().to_string_lossy().to_string(),
				clean: Some(true),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(text.contains("1 schema"), "expected '1 schema' in: {text}");

		let select = server
			.run_query(Parameters(ExecArgs {
				query: "SELECT * FROM user".into(),
			}))
			.await
			.unwrap();
		assert!(result_text(&select).contains("empty"));
	}

	#[tokio::test]
	async fn should_load_project_categorize_and_report() {
		let dir = TempDir::new().unwrap();
		fs::create_dir_all(dir.path().join("schema")).unwrap();
		fs::create_dir_all(dir.path().join("migrations")).unwrap();
		fs::create_dir_all(dir.path().join("functions")).unwrap();
		fs::create_dir_all(dir.path().join("examples")).unwrap();
		fs::write(
			dir.path().join("schema/tables.surql"),
			"DEFINE TABLE user SCHEMAFULL;\n\
			 DEFINE FIELD name ON user TYPE string;",
		)
		.unwrap();
		fs::write(
			dir.path().join("migrations/001_init.surql"),
			"CREATE user:alice SET name = 'Alice';",
		)
		.unwrap();
		fs::write(
			dir.path().join("functions/greet.surql"),
			"DEFINE FUNCTION fn::greet() { RETURN 'hi'; };",
		)
		.unwrap();
		fs::write(
			dir.path().join("examples/demo.surql"),
			"SELECT * FROM user;",
		)
		.unwrap();

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();
		let result = server
			.load_project(Parameters(LoadProjectArgs {
				path: dir.path().to_string_lossy().to_string(),
				clean: Some(true),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(text.contains("1 schema"), "expected 1 schema in: {text}");
		assert!(
			text.contains("1 migrations"),
			"expected 1 migrations in: {text}"
		);
		assert!(
			text.contains("1 functions"),
			"expected 1 functions in: {text}"
		);
		assert!(
			text.contains("1 examples"),
			"expected 1 examples in: {text}"
		);
	}

	#[tokio::test]
	async fn should_load_project_example_errors_become_warnings() {
		let dir = TempDir::new().unwrap();
		fs::create_dir_all(dir.path().join("examples")).unwrap();
		fs::write(dir.path().join("examples/bad.surql"), "NOT VALID SQL !!!").unwrap();

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();
		let result = server
			.load_project(Parameters(LoadProjectArgs {
				path: dir.path().to_string_lossy().to_string(),
				clean: Some(true),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(
			text.contains("Warnings"),
			"example errors should be warnings: {text}"
		);
		assert!(
			!text.contains("**Errors"),
			"example errors should NOT appear as errors: {text}"
		);
	}

	#[tokio::test]
	async fn should_load_project_inject_overwrite_for_schema() {
		let dir = TempDir::new().unwrap();
		fs::create_dir_all(dir.path().join("schema")).unwrap();
		fs::write(
			dir.path().join("schema/tables.surql"),
			"DEFINE TABLE user SCHEMAFULL;",
		)
		.unwrap();

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();

		// Load twice -- second load should succeed due to OVERWRITE injection
		server
			.load_project(Parameters(LoadProjectArgs {
				path: dir.path().to_string_lossy().to_string(),
				clean: Some(true),
			}))
			.await
			.unwrap();
		let result = server
			.load_project(Parameters(LoadProjectArgs {
				path: dir.path().to_string_lossy().to_string(),
				clean: Some(false),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(
			text.contains("1 schema"),
			"second load with OVERWRITE should succeed: {text}"
		);
		assert!(
			!text.contains("**Errors"),
			"second load should not have errors: {text}"
		);
	}

	#[tokio::test]
	async fn should_load_project_reject_nonexistent_dir() {
		let server = SurqlMcp::new().await.unwrap();
		let result = server
			.load_project(Parameters(LoadProjectArgs {
				path: "/nonexistent/path".into(),
				clean: None,
			}))
			.await
			.unwrap();
		assert!(result.is_error == Some(true));
	}

	#[tokio::test]
	async fn should_schema_return_db_info() {
		let server = SurqlMcp::new().await.unwrap();
		server
			.run_query(Parameters(ExecArgs {
				query: "DEFINE TABLE user SCHEMAFULL".into(),
			}))
			.await
			.unwrap();

		let result = server.schema().await.unwrap();
		let text = result_text(&result);
		assert!(text.contains("user"), "expected user in schema: {text}");
	}

	#[tokio::test]
	async fn should_describe_return_table_info() {
		let server = SurqlMcp::new().await.unwrap();
		server
			.run_query(Parameters(ExecArgs {
				query: "DEFINE TABLE post SCHEMAFULL; \
				 DEFINE FIELD title ON post TYPE string"
					.into(),
			}))
			.await
			.unwrap();

		let result = server
			.describe(Parameters(DescribeArgs {
				table: "post".into(),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(text.contains("post"), "expected post in: {text}");
		assert!(text.contains("title"), "expected title field in: {text}");
	}

	#[tokio::test]
	async fn should_reset_clear_all_data() {
		let server = SurqlMcp::new().await.unwrap();
		server
			.run_query(Parameters(ExecArgs {
				query: "CREATE user:alice SET name = 'Alice'".into(),
			}))
			.await
			.unwrap();

		server.reset().await.unwrap();

		let result = server
			.run_query(Parameters(ExecArgs {
				query: "SELECT * FROM user".into(),
			}))
			.await
			.unwrap();
		assert!(
			result.is_error == Some(true),
			"expected error after reset (table gone)"
		);
	}

	#[tokio::test]
	async fn should_load_file_single() {
		let dir = TempDir::new().unwrap();
		let file = dir.path().join("schema.surql");
		fs::write(&file, "DEFINE TABLE test SCHEMAFULL;").unwrap();

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();
		let result = server
			.load_file(Parameters(LoadFileArgs {
				path: file.to_string_lossy().to_string(),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(text.contains("Applied"), "expected Applied in: {text}");
	}

	#[tokio::test]
	async fn should_load_file_report_error_for_missing() {
		let server = SurqlMcp::new().await.unwrap();
		let result = server
			.load_file(Parameters(LoadFileArgs {
				path: "/nonexistent.surql".into(),
			}))
			.await
			.unwrap();
		assert!(result.is_error == Some(true));
	}

	#[tokio::test]
	async fn should_reject_describe_with_injection() {
		let server = SurqlMcp::new().await.unwrap();
		let result = server
			.describe(Parameters(DescribeArgs {
				table: "user; REMOVE DATABASE default".into(),
			}))
			.await
			.unwrap();
		assert!(
			result.is_error == Some(true),
			"should reject table name with injection"
		);
	}

	#[test]
	fn should_categorize_files_by_directory() {
		let schema = PathBuf::from("/project/schema.surql");
		let schema_dir = PathBuf::from("/project/schema/tables.surql");
		let migrations = PathBuf::from("/project/migrations/001.surql");
		let example = PathBuf::from("/project/examples/demo.surql");
		let seed = PathBuf::from("/project/seed/data.surql");
		let func = PathBuf::from("/project/functions/auth.surql");
		let func_file = PathBuf::from("/project/functions.surql");
		let other = PathBuf::from("/project/queries.surql");

		assert_eq!(classify_file(&schema), FileCategory::Schema);
		assert_eq!(classify_file(&schema_dir), FileCategory::Schema);
		assert_eq!(classify_file(&migrations), FileCategory::Migration);
		assert_eq!(classify_file(&example), FileCategory::Example);
		assert_eq!(classify_file(&seed), FileCategory::Example);
		assert_eq!(classify_file(&func), FileCategory::Function);
		assert_eq!(classify_file(&func_file), FileCategory::Function);
		assert_eq!(classify_file(&other), FileCategory::General);
	}

	#[test]
	fn should_inject_overwrite_into_define_statements() {
		let input = "\
DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD name ON user TYPE string;
DEFINE INDEX user_name ON user FIELDS name UNIQUE;
DEFINE FUNCTION fn::greet() { RETURN 'hi'; };";

		let result = inject_overwrite(input);
		assert!(
			result.contains("DEFINE TABLE OVERWRITE user"),
			"expected OVERWRITE after DEFINE TABLE: {result}"
		);
		assert!(
			result.contains("DEFINE FIELD OVERWRITE name"),
			"expected OVERWRITE after DEFINE FIELD: {result}"
		);
		assert!(
			result.contains("DEFINE INDEX OVERWRITE user_name"),
			"expected OVERWRITE after DEFINE INDEX: {result}"
		);
		assert!(
			result.contains("DEFINE FUNCTION OVERWRITE fn::greet"),
			"expected OVERWRITE after DEFINE FUNCTION: {result}"
		);
	}

	#[test]
	fn should_not_double_inject_overwrite() {
		let input = "DEFINE TABLE OVERWRITE user SCHEMAFULL;";
		let result = inject_overwrite(input);
		assert_eq!(
			result.matches("OVERWRITE").count(),
			1,
			"should not double-inject OVERWRITE: {result}"
		);
	}

	#[test]
	fn should_not_inject_overwrite_when_if_not_exists() {
		let input = "DEFINE TABLE IF NOT EXISTS user SCHEMAFULL;";
		let result = inject_overwrite(input);
		assert!(
			!result.contains("OVERWRITE"),
			"should not inject OVERWRITE when IF NOT EXISTS is present: {result}"
		);
	}

	#[test]
	fn should_inject_overwrite_preserve_indentation() {
		let input = "  \tDEFINE TABLE user SCHEMAFULL;";
		let result = inject_overwrite(input);
		assert!(
			result.starts_with("  \tDEFINE TABLE OVERWRITE user"),
			"should preserve leading whitespace: {result}"
		);
	}

	#[test]
	fn should_inject_overwrite_all_supported_keywords() {
		let input = "\
DEFINE TABLE t1;
DEFINE FIELD f1 ON t1 TYPE string;
DEFINE INDEX i1 ON t1 FIELDS f1;
DEFINE FUNCTION fn::x() { RETURN 1; };
DEFINE EVENT e1 ON t1 WHEN true THEN {};
DEFINE ANALYZER a1 TOKENIZERS blank;
DEFINE PARAM $p VALUE 1;";

		let result = inject_overwrite(input);
		assert!(result.contains("DEFINE TABLE OVERWRITE"), "{result}");
		assert!(result.contains("DEFINE FIELD OVERWRITE"), "{result}");
		assert!(result.contains("DEFINE INDEX OVERWRITE"), "{result}");
		assert!(result.contains("DEFINE FUNCTION OVERWRITE"), "{result}");
		assert!(result.contains("DEFINE EVENT OVERWRITE"), "{result}");
		assert!(result.contains("DEFINE ANALYZER OVERWRITE"), "{result}");
		assert!(result.contains("DEFINE PARAM OVERWRITE"), "{result}");
	}

	#[test]
	fn should_inject_overwrite_case_insensitive() {
		let input = "define table user SCHEMAFULL;";
		let result = inject_overwrite(input);
		assert!(
			result.contains("define table OVERWRITE user"),
			"should handle case-insensitive DEFINE: {result}"
		);
	}

	#[test]
	fn should_not_inject_overwrite_in_line_comments() {
		let input = "-- DEFINE TABLE user SCHEMAFULL;\nDEFINE TABLE post;";
		let result = inject_overwrite(input);
		assert!(
			result.contains("-- DEFINE TABLE user SCHEMAFULL;"),
			"should not inject OVERWRITE inside line comment: {result}"
		);
		assert!(
			result.contains("DEFINE TABLE OVERWRITE post"),
			"should still inject OVERWRITE outside comment: {result}"
		);
	}

	#[test]
	fn should_not_inject_overwrite_in_block_comments() {
		let input = "/*\nDEFINE TABLE user SCHEMAFULL;\n*/\nDEFINE TABLE post;";
		let result = inject_overwrite(input);
		assert!(
			!result.contains("DEFINE TABLE OVERWRITE user"),
			"should not inject OVERWRITE inside block comment: {result}"
		);
		assert!(
			result.contains("DEFINE TABLE OVERWRITE post"),
			"should still inject OVERWRITE after block comment: {result}"
		);
	}

	#[tokio::test]
	async fn should_read_overshift_manifest() {
		let dir = TempDir::new().unwrap();
		fs::write(
			dir.path().join("manifest.toml"),
			"[meta]\nns = \"myapp\"\ndb = \"main\"\nsystem_db = \"_system\"\n\n\
			 [[modules]]\nname = \"auth\"\npath = \"schema/auth\"\ndepends_on = []\n",
		)
		.unwrap();
		fs::create_dir_all(dir.path().join("migrations")).unwrap();
		fs::write(
			dir.path().join("migrations/v001_init.surql"),
			"DEFINE TABLE user;",
		)
		.unwrap();

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();
		let result = server
			.manifest(Parameters(ManifestArgs {
				path: dir.path().to_string_lossy().to_string(),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(text.contains("myapp"), "expected ns in: {text}");
		assert!(text.contains("main"), "expected db in: {text}");
		assert!(text.contains("auth"), "expected module in: {text}");
		assert!(
			text.contains("1 migration"),
			"expected migrations in: {text}"
		);
	}

	#[tokio::test]
	async fn should_reject_missing_manifest() {
		let server = SurqlMcp::new().await.unwrap();
		let result = server
			.manifest(Parameters(ManifestArgs {
				path: "/nonexistent".into(),
			}))
			.await
			.unwrap();
		assert!(result.is_error == Some(true));
	}

	#[tokio::test]
	async fn should_compare_detect_missing_table() {
		let server = SurqlMcp::new().await.unwrap();
		server
			.run_query(Parameters(ExecArgs {
				query: "DEFINE TABLE user SCHEMAFULL".into(),
			}))
			.await
			.unwrap();

		let expected_json = serde_json::json!({
			"tables": {
				"user": "DEFINE TABLE user TYPE NORMAL SCHEMAFULL",
				"post": "DEFINE TABLE post TYPE NORMAL SCHEMAFULL"
			},
			"functions": {}
		});

		let result = server
			.compare(Parameters(CompareArgs {
				expected_json: expected_json.to_string(),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(
			text.contains("post") && text.contains("missing"),
			"expected missing table 'post' in diff: {text}"
		);
	}

	#[tokio::test]
	async fn should_compare_return_match_when_identical() {
		let server = SurqlMcp::new().await.unwrap();
		server
			.run_query(Parameters(ExecArgs {
				query: "DEFINE TABLE user SCHEMAFULL; \
				 DEFINE TABLE post SCHEMAFULL"
					.into(),
			}))
			.await
			.unwrap();

		let schema_result = server.schema().await.unwrap();
		let schema_text = result_text(&schema_result);
		let json_start = schema_text.find('{').expect("schema should contain JSON");
		let json_end = schema_text.rfind('}').expect("schema should contain JSON") + 1;
		let raw_json = &schema_text[json_start..json_end];

		let result = server
			.compare(Parameters(CompareArgs {
				expected_json: raw_json.to_string(),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(
			text.contains("Schema matches"),
			"expected 'Schema matches' for identical schemas: {text}"
		);
	}

	#[tokio::test]
	async fn should_verify_matching_project() {
		let dir = TempDir::new().unwrap();
		fs::write(
			dir.path().join("manifest.toml"),
			"[meta]\nns = \"test\"\ndb = \"main\"\nsystem_db = \"_system\"\n\n\
			 [[modules]]\nname = \"core\"\npath = \"schema/core\"\n",
		)
		.unwrap();
		fs::create_dir_all(dir.path().join("schema/core")).unwrap();
		fs::write(
			dir.path().join("schema/core/tables.surql"),
			"DEFINE TABLE user SCHEMAFULL;\n\
			 DEFINE FIELD name ON user TYPE string;",
		)
		.unwrap();

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();
		let result = server
			.verify(Parameters(VerifyArgs {
				verify_only: None,
				path: dir.path().to_string_lossy().to_string(),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(
			text.contains("Schema matches"),
			"expected matching schemas in: {text}"
		);
		assert!(text.contains("1 module(s)"), "expected 1 module in: {text}");
	}

	#[tokio::test]
	async fn should_verify_reject_missing_manifest() {
		let server = SurqlMcp::new().await.unwrap();
		let result = server
			.verify(Parameters(VerifyArgs {
				verify_only: None,
				path: "/nonexistent".into(),
			}))
			.await
			.unwrap();
		assert!(result.is_error == Some(true));
	}

	#[tokio::test]
	async fn should_check_valid_file() {
		let dir = TempDir::new().unwrap();
		let file = dir.path().join("schema.surql");
		fs::write(&file, "DEFINE TABLE user SCHEMAFULL;").unwrap();

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();
		let result = server
			.check(Parameters(CheckArgs {
				path: file.to_string_lossy().to_string(),
				recursive: None,
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(
			text.contains("1 file checked") && text.contains("0 errors"),
			"expected no errors for valid file: {text}"
		);
	}

	#[tokio::test]
	async fn should_check_invalid_file_report_errors() {
		let dir = TempDir::new().unwrap();
		let file = dir.path().join("broken.surql");
		fs::write(&file, "SELEC * FORM user;").unwrap();

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();
		let result = server
			.check(Parameters(CheckArgs {
				path: file.to_string_lossy().to_string(),
				recursive: None,
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(
			!text.contains("0 errors"),
			"expected errors for invalid file: {text}"
		);
	}

	#[tokio::test]
	async fn should_check_directory_recursively() {
		let dir = TempDir::new().unwrap();
		let sub = dir.path().join("schemas");
		fs::create_dir_all(&sub).unwrap();
		fs::write(sub.join("a.surql"), "DEFINE TABLE a;").unwrap();
		fs::write(dir.path().join("b.surql"), "DEFINE TABLE b;").unwrap();

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();
		let result = server
			.check(Parameters(CheckArgs {
				path: dir.path().to_string_lossy().to_string(),
				recursive: Some(true),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(
			text.contains("2 files checked"),
			"expected 2 files checked: {text}"
		);
	}

	#[tokio::test]
	async fn should_check_nonrecursive_skip_subdirs() {
		let dir = TempDir::new().unwrap();
		let sub = dir.path().join("schemas");
		fs::create_dir_all(&sub).unwrap();
		fs::write(sub.join("a.surql"), "DEFINE TABLE a;").unwrap();
		fs::write(dir.path().join("b.surql"), "DEFINE TABLE b;").unwrap();

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();
		let result = server
			.check(Parameters(CheckArgs {
				path: dir.path().to_string_lossy().to_string(),
				recursive: Some(false),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(
			text.contains("1 file checked"),
			"expected 1 file checked (non-recursive): {text}"
		);
	}

	#[tokio::test]
	async fn should_check_reject_nonexistent_path() {
		let server = SurqlMcp::new().await.unwrap();
		let result = server
			.check(Parameters(CheckArgs {
				path: "/nonexistent/path.surql".into(),
				recursive: None,
			}))
			.await
			.unwrap();
		assert!(
			result.is_error == Some(true),
			"expected error for nonexistent path"
		);
	}

	#[tokio::test]
	async fn should_check_empty_directory() {
		let dir = TempDir::new().unwrap();

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();
		let result = server
			.check(Parameters(CheckArgs {
				path: dir.path().to_string_lossy().to_string(),
				recursive: None,
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(
			text.contains("No .surql files found"),
			"expected no files found: {text}"
		);
	}

	fn write_graph_schema(dir: &std::path::Path) {
		fs::create_dir_all(dir.join("schema")).unwrap();
		fs::write(
			dir.join("schema/tables.surql"),
			"DEFINE TABLE user SCHEMAFULL;\n\
			 DEFINE TABLE post SCHEMAFULL;\n\
			 DEFINE TABLE comment SCHEMAFULL;\n\
			 DEFINE FIELD author ON post TYPE record<user>;\n\
			 DEFINE FIELD post ON comment TYPE record<post>;\n\
			 DEFINE FIELD author ON comment TYPE record<user>;\n",
		)
		.unwrap();
	}

	#[tokio::test]
	async fn should_graph_affected_find_dependents() {
		let dir = TempDir::new().unwrap();
		write_graph_schema(dir.path());

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();
		let result = server
			.graph_affected(Parameters(GraphAffectedArgs {
				table: "user".into(),
				schema_path: dir.path().to_string_lossy().to_string(),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(
			text.contains("comment.author"),
			"expected comment.author in: {text}"
		);
		assert!(
			text.contains("post.author"),
			"expected post.author in: {text}"
		);
	}

	#[tokio::test]
	async fn should_graph_affected_report_no_dependents() {
		let dir = TempDir::new().unwrap();
		write_graph_schema(dir.path());

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();
		let result = server
			.graph_affected(Parameters(GraphAffectedArgs {
				table: "comment".into(),
				schema_path: dir.path().to_string_lossy().to_string(),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(
			text.contains("No tables reference"),
			"expected no references for leaf table: {text}"
		);
	}

	#[tokio::test]
	async fn should_graph_affected_reject_missing_table() {
		let dir = TempDir::new().unwrap();
		write_graph_schema(dir.path());

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();
		let result = server
			.graph_affected(Parameters(GraphAffectedArgs {
				table: "nonexistent".into(),
				schema_path: dir.path().to_string_lossy().to_string(),
			}))
			.await
			.unwrap();
		assert!(
			result.is_error == Some(true),
			"expected error for nonexistent table"
		);
	}

	#[tokio::test]
	async fn should_graph_traverse_forward() {
		let dir = TempDir::new().unwrap();
		write_graph_schema(dir.path());

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();
		let result = server
			.graph_traverse(Parameters(GraphTraverseArgs {
				table: "comment".into(),
				schema_path: dir.path().to_string_lossy().to_string(),
				depth: None,
				direction: Some("forward".into()),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(text.contains("post"), "expected post in traversal: {text}");
		assert!(text.contains("user"), "expected user in traversal: {text}");
	}

	#[tokio::test]
	async fn should_graph_traverse_reverse() {
		let dir = TempDir::new().unwrap();
		write_graph_schema(dir.path());

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();
		let result = server
			.graph_traverse(Parameters(GraphTraverseArgs {
				table: "user".into(),
				schema_path: dir.path().to_string_lossy().to_string(),
				depth: None,
				direction: Some("reverse".into()),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(
			text.contains("post.author"),
			"expected post.author in reverse: {text}"
		);
		assert!(
			text.contains("comment.author"),
			"expected comment.author in reverse: {text}"
		);
	}

	#[tokio::test]
	async fn should_graph_traverse_reject_invalid_direction() {
		let dir = TempDir::new().unwrap();
		write_graph_schema(dir.path());

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();
		let result = server
			.graph_traverse(Parameters(GraphTraverseArgs {
				table: "user".into(),
				schema_path: dir.path().to_string_lossy().to_string(),
				depth: None,
				direction: Some("sideways".into()),
			}))
			.await
			.unwrap();
		assert!(
			result.is_error == Some(true),
			"expected error for invalid direction"
		);
	}

	#[tokio::test]
	async fn should_graph_siblings_find_shared_targets() {
		let dir = TempDir::new().unwrap();
		write_graph_schema(dir.path());

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();
		let result = server
			.graph_siblings(Parameters(GraphSiblingsArgs {
				table: "post".into(),
				schema_path: dir.path().to_string_lossy().to_string(),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(
			text.contains("comment"),
			"expected comment as sibling of post (both link to user): {text}"
		);
	}

	#[tokio::test]
	async fn should_graph_siblings_report_no_siblings() {
		let dir = TempDir::new().unwrap();
		fs::create_dir_all(dir.path().join("schema")).unwrap();
		fs::write(
			dir.path().join("schema/tables.surql"),
			"DEFINE TABLE solo SCHEMAFULL;\n\
			 DEFINE FIELD name ON solo TYPE string;\n",
		)
		.unwrap();

		let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
			.await
			.unwrap();
		let result = server
			.graph_siblings(Parameters(GraphSiblingsArgs {
				table: "solo".into(),
				schema_path: dir.path().to_string_lossy().to_string(),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(
			text.contains("shares no record<> targets"),
			"expected no siblings for isolated table: {text}"
		);
	}
}
