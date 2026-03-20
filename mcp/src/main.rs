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
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct DescribeArgs {
	#[schemars(description = "Table name to describe")]
	table: String,
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
		description = "Load .surql files from a directory as migrations (sorted by name, recursive)"
	)]
	async fn load_project(
		&self,
		Parameters(args): Parameters<LoadProjectArgs>,
	) -> Result<CallToolResult, rmcp::ErrorData> {
		let dir = PathBuf::from(&args.path);
		if !dir.is_dir() {
			return error_result(format!("Not a directory: {}", args.path));
		}

		let mut surql_files = Vec::new();
		collect_surql_files(&dir, &mut surql_files);
		surql_files.sort();

		if surql_files.is_empty() {
			return Ok(CallToolResult::success(vec![Content::text(
				"No .surql files found",
			)]));
		}

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
			"Loaded {applied}/{} files from `{}`",
			surql_files.len(),
			args.path
		);
		if !errors.is_empty() {
			output.push_str(&format!("\n\n**Errors:**\n{}", errors.join("\n")));
		}
		Ok(CallToolResult::success(vec![Content::text(output)]))
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
