use rmcp::{
	ServiceExt,
	handler::server::wrapper::Parameters,
	model::{CallToolResult, Content, Implementation, ServerCapabilities, ServerInfo},
	schemars, tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use surrealdb::{Surreal, engine::local::Mem};
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ExecArgs {
	#[schemars(description = "SurrealQL query to execute")]
	query: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct LoadProjectArgs {
	#[schemars(description = "Path to directory containing .surql files")]
	path: String,
	#[schemars(description = "Reset database before loading (default: true)")]
	clean: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct LoadFileArgs {
	#[schemars(description = "Path to a single .surql file to execute")]
	path: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct DescribeArgs {
	#[schemars(description = "Table name to describe")]
	table: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ManifestArgs {
	#[schemars(description = "Path to directory containing manifest.toml (overshift project)")]
	path: String,
}

fn error_result(msg: String) -> Result<CallToolResult, rmcp::ErrorData> {
	Ok(CallToolResult::error(vec![Content::text(msg)]))
}

#[derive(Clone)]
struct SurqlMcp {
	db: Arc<RwLock<Surreal<surrealdb::engine::local::Db>>>,
	tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
}

#[tool_router]
impl SurqlMcp {
	async fn new() -> anyhow::Result<Self> {
		let db = Surreal::new::<Mem>(()).await?;
		db.use_ns("default").use_db("default").await?;
		tracing::info!("SurrealDB playground started");
		Ok(Self {
			db: Arc::new(RwLock::new(db)),
			tool_router: Self::tool_router(),
		})
	}

	#[tool(
		name = "exec",
		description = "Execute a SurrealQL query and return the result as JSON"
	)]
	async fn exec(
		&self,
		Parameters(args): Parameters<ExecArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
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
		description = "Load .surql files from a directory into the database. Resets DB first by default. Files are loaded in priority order: migrations/ and schema files first, then examples/"
	)]
	async fn load_project(
		&self,
		Parameters(args): Parameters<LoadProjectArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
		let dir = PathBuf::from(&args.path);
		if !dir.is_dir() {
			return error_result(format!("Not a directory: {}", args.path));
		}

		let clean = args.clean.unwrap_or(true);
		if clean {
			let db = self.db.read().await;
			db.query("REMOVE DATABASE default").await.ok();
			db.use_ns("default").use_db("default").await.ok();
		}

		let mut surql_files = Vec::new();
		collect_surql_files(&dir, &mut surql_files);

		if surql_files.is_empty() {
			return Ok(CallToolResult::success(vec![Content::text(
				"No .surql files found",
			)]));
		}

		surql_files.sort_by_key(|p| file_load_priority(p));

		let db = self.db.read().await;
		let mut applied = 0;
		let mut errors = Vec::new();

		for path in &surql_files {
			let content = match std::fs::read_to_string(path) {
				Ok(c) => c,
				Err(e) => {
					errors.push(format!("{}: {e}", path.display()));
					continue;
				}
			};
			match db.query(&content).await {
				Ok(response) => match response.check() {
					Ok(_) => applied += 1,
					Err(e) => errors.push(format!("{}: {e}", path.display())),
				},
				Err(e) => errors.push(format!("{}: {e}", path.display())),
			}
		}

		let mut output = format!(
			"Loaded {applied}/{} files from `{}`{}",
			surql_files.len(),
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
		Ok(CallToolResult::success(vec![Content::text(output)]))
	}

	#[tool(
		name = "load_file",
		description = "Execute a single .surql file against the database"
	)]
	async fn load_file(
		&self,
		Parameters(args): Parameters<LoadFileArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
		let path = PathBuf::from(&args.path);
		let content = match std::fs::read_to_string(&path) {
			Ok(c) => c,
			Err(e) => return error_result(format!("Cannot read {}: {e}", args.path)),
		};
		let db = self.db.read().await;
		match db.query(&content).await {
			Ok(response) => match response.check() {
				Ok(_) => Ok(CallToolResult::success(vec![Content::text(format!(
					"Executed `{}`",
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
	async fn schema(&self) -> Result<CallToolResult, rmcp::ErrorData> {
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
	async fn describe(
		&self,
		Parameters(args): Parameters<DescribeArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
		let db = self.db.read().await;
		let query = format!("INFO FOR TABLE {}", args.table);
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
		description = "Read an overshift manifest.toml and show project configuration (namespace, database, modules, migrations)"
	)]
	async fn manifest(
		&self,
		Parameters(args): Parameters<ManifestArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
		let manifest = match overshift::Manifest::load(&args.path) {
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
				output.push_str(&format!("- `{}` → `{}`{deps}\n", m.name, m.path));
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
		description = "Load an overshift project into the playground DB: applies schema modules then migrations in order"
	)]
	async fn load_manifest(
		&self,
		Parameters(args): Parameters<ManifestArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
		let manifest = match overshift::Manifest::load(&args.path) {
			Ok(m) => m,
			Err(e) => return error_result(format!("Cannot load manifest: {e}")),
		};

		// Reset DB
		let db = self.db.read().await;
		db.query("REMOVE DATABASE default").await.ok();
		db.use_ns(&manifest.meta.ns)
			.use_db(&manifest.meta.db)
			.await
			.ok();

		let mut applied = 0;
		let mut errors = Vec::new();

		// Apply schema modules first
		let modules = overshift::schema::load_schema_modules(&manifest).unwrap_or_default();
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
		let migrations = overshift::migration::discover_migrations(
			manifest.root_path().unwrap_or(std::path::Path::new(".")),
		)
		.unwrap_or_default();
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
			"Loaded overshift project `{}` (NS={}, DB={})\n{} schema module(s) + {} migration(s) = {applied} applied",
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

	#[tool(name = "reset", description = "Clear the database and start fresh")]
	async fn reset(&self) -> Result<CallToolResult, rmcp::ErrorData> {
		let db = self.db.read().await;
		// REMOVE DATABASE may fail if it doesn't exist yet — safe to ignore
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
				"SurrealQL playground: execute queries, load schema files, explore database".into(),
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

/// Assign load priority based on parent directories and file name.
/// migrations/ first, then root schema/function files, then everything else, examples last.
fn file_load_priority(path: &std::path::Path) -> (u8, std::path::PathBuf) {
	let parent_names: Vec<String> = path
		.ancestors()
		.filter_map(|a| a.file_name())
		.map(|n| n.to_string_lossy().to_lowercase())
		.collect();

	let in_dir = |name: &str| parent_names.iter().any(|d| d.contains(name));

	let file_stem = path
		.file_stem()
		.and_then(|n| n.to_str())
		.unwrap_or("")
		.to_lowercase();

	let priority = if in_dir("example") || in_dir("seed") || in_dir("test") {
		4
	} else if in_dir("migration") {
		0
	} else if file_stem.contains("schema") {
		1
	} else if file_stem.contains("function") {
		2
	} else {
		3
	};
	(priority, path.to_path_buf())
}

fn collect_surql_files(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
	let entries = match std::fs::read_dir(dir) {
		Ok(e) => e,
		Err(_) => return,
	};
	for entry in entries.filter_map(|e| e.ok()) {
		let path = entry.path();
		if path.is_dir() {
			let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
			if matches!(
				name,
				"target" | "node_modules" | ".git" | "build" | "fixtures" | "dist" | ".cache"
			) || name.starts_with('.')
			{
				continue;
			}
			collect_surql_files(&path, out);
		} else if path.extension().is_some_and(|ext| ext == "surql") {
			out.push(path);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use rmcp::handler::server::wrapper::Parameters;
	use std::fs;
	use tempfile::TempDir;

	#[tokio::test]
	async fn should_start_and_exec_return_query() {
		let server = SurqlMcp::new().await.unwrap();
		let result = server
			.exec(Parameters(ExecArgs {
				query: "RETURN 42".into(),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(text.contains("42"), "expected 42 in: {text}");
	}

	#[tokio::test]
	async fn should_exec_create_and_select() {
		let server = SurqlMcp::new().await.unwrap();
		server
			.exec(Parameters(ExecArgs {
				query: "CREATE user:alice SET name = 'Alice'".into(),
			}))
			.await
			.unwrap();

		let result = server
			.exec(Parameters(ExecArgs {
				query: "SELECT * FROM user".into(),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(text.contains("Alice"), "expected Alice in: {text}");
		assert!(text.contains("1 row"), "expected 1 row in: {text}");
	}

	#[tokio::test]
	async fn should_exec_return_error_for_missing_table() {
		let server = SurqlMcp::new().await.unwrap();
		let result = server
			.exec(Parameters(ExecArgs {
				query: "SELECT * FROM nonexistent".into(),
			}))
			.await
			.unwrap();
		assert!(
			result.is_error == Some(true),
			"expected error for nonexistent table"
		);
	}

	#[tokio::test]
	async fn should_exec_report_syntax_error() {
		let server = SurqlMcp::new().await.unwrap();
		let result = server
			.exec(Parameters(ExecArgs {
				query: "NOT VALID SQL !!!".into(),
			}))
			.await
			.unwrap();
		assert!(result.is_error == Some(true));
	}

	#[tokio::test]
	async fn should_load_project_from_directory() {
		let dir = TempDir::new().unwrap();
		fs::write(
			dir.path().join("001_schema.surql"),
			"DEFINE TABLE user SCHEMAFULL; DEFINE FIELD name ON user TYPE string;",
		)
		.unwrap();

		let server = SurqlMcp::new().await.unwrap();
		let result = server
			.load_project(Parameters(LoadProjectArgs {
				path: dir.path().to_string_lossy().to_string(),
				clean: Some(true),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(text.contains("Loaded 1/1"), "expected loaded in: {text}");

		let select = server
			.exec(Parameters(ExecArgs {
				query: "SELECT * FROM user".into(),
			}))
			.await
			.unwrap();
		assert!(result_text(&select).contains("empty"));
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
			.exec(Parameters(ExecArgs {
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
			.exec(Parameters(ExecArgs {
				query: "DEFINE TABLE post SCHEMAFULL; DEFINE FIELD title ON post TYPE string"
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
			.exec(Parameters(ExecArgs {
				query: "CREATE user:alice SET name = 'Alice'".into(),
			}))
			.await
			.unwrap();

		server.reset().await.unwrap();

		let result = server
			.exec(Parameters(ExecArgs {
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

		let server = SurqlMcp::new().await.unwrap();
		let result = server
			.load_file(Parameters(LoadFileArgs {
				path: file.to_string_lossy().to_string(),
			}))
			.await
			.unwrap();
		let text = result_text(&result);
		assert!(text.contains("Executed"), "expected executed in: {text}");
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

	#[test]
	fn should_prioritize_migrations_over_examples() {
		let migrations = PathBuf::from("/project/migrations/001.surql");
		let schema = PathBuf::from("/project/schema.surql");
		let example = PathBuf::from("/project/examples/demo.surql");
		let other = PathBuf::from("/project/queries.surql");

		assert!(file_load_priority(&migrations) < file_load_priority(&schema));
		assert!(file_load_priority(&schema) < file_load_priority(&other));
		assert!(file_load_priority(&other) < file_load_priority(&example));
	}

	#[tokio::test]
	async fn should_read_overshift_manifest() {
		let dir = TempDir::new().unwrap();
		fs::write(
			dir.path().join("manifest.toml"),
			r#"
[meta]
ns = "myapp"
db = "main"
system_db = "_system"

[[modules]]
name = "auth"
path = "schema/auth"
depends_on = []
"#,
		)
		.unwrap();
		fs::create_dir_all(dir.path().join("migrations")).unwrap();
		fs::write(
			dir.path().join("migrations/v001_init.surql"),
			"DEFINE TABLE user;",
		)
		.unwrap();

		let server = SurqlMcp::new().await.unwrap();
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

	fn result_text(result: &CallToolResult) -> String {
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt()
		.with_env_filter(
			tracing_subscriber::EnvFilter::from_default_env()
				.add_directive(tracing::Level::INFO.into()),
		)
		.with_writer(std::io::stderr)
		.init();

	let server = SurqlMcp::new().await?;
	let transport = rmcp::transport::io::stdio();
	let ct = server.serve(transport).await?;
	ct.waiting().await?;
	Ok(())
}
