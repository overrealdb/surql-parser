use rmcp::handler::server::wrapper::Parameters;
use std::fs;
use surql_mcp::{
	CompareArgs, DescribeArgs, ExecArgs, ManifestArgs, SurqlMcp, VerifyArgs, result_text,
};
use tempfile::TempDir;

fn create_manifest(dir: &std::path::Path, ns: &str, db: &str, modules: &[(&str, &str)]) {
	let mut toml = format!("[meta]\nns = \"{ns}\"\ndb = \"{db}\"\nsystem_db = \"_system\"\n");
	for (name, path) in modules {
		toml.push_str(&format!(
			"\n[[modules]]\nname = \"{name}\"\npath = \"{path}\"\n"
		));
	}
	fs::write(dir.join("manifest.toml"), toml).unwrap();
}

#[tokio::test]
async fn should_full_lifecycle_load_verify_compare() {
	let dir = TempDir::new().unwrap();

	create_manifest(dir.path(), "lifecycle", "main", &[("core", "schema/core")]);

	fs::create_dir_all(dir.path().join("schema/core")).unwrap();
	fs::write(
		dir.path().join("schema/core/tables.surql"),
		"DEFINE TABLE user SCHEMAFULL;\n\
		 DEFINE FIELD name ON user TYPE string;\n\
		 DEFINE FIELD email ON user TYPE string;",
	)
	.unwrap();

	fs::create_dir_all(dir.path().join("migrations")).unwrap();
	fs::write(
		dir.path().join("migrations/v001_seed.surql"),
		"DEFINE INDEX idx_user_email ON user FIELDS email UNIQUE;",
	)
	.unwrap();

	let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
		.await
		.unwrap();

	// Step 1: load_manifest applies schema + migrations to playground
	let load_result = server
		.load_manifest(Parameters(ManifestArgs {
			path: dir.path().to_string_lossy().to_string(),
		}))
		.await
		.unwrap();
	let load_text = result_text(&load_result);
	assert!(
		load_text.contains("1 schema module(s)"),
		"expected 1 schema module in load output: {load_text}"
	);
	assert!(
		load_text.contains("1 migration(s)"),
		"expected 1 migration in load output: {load_text}"
	);

	// Step 2: verify confirms playground and shadow match
	let verify_result = server
		.verify(Parameters(VerifyArgs {
			verify_only: None,
			path: dir.path().to_string_lossy().to_string(),
		}))
		.await
		.unwrap();
	let verify_text = result_text(&verify_result);
	assert!(
		verify_text.contains("Schema matches"),
		"expected schemas to match after clean load+verify: {verify_text}"
	);
	assert!(
		verify_text.contains("1 module(s)"),
		"expected module count in verify: {verify_text}"
	);
	assert!(
		verify_text.contains("1 migration(s)"),
		"expected migration count in verify: {verify_text}"
	);

	// Step 3: schema shows the user table
	let schema_result = server.schema().await.unwrap();
	let schema_text = result_text(&schema_result);
	assert!(
		schema_text.contains("user"),
		"expected user table in schema: {schema_text}"
	);
}

#[tokio::test]
async fn should_detect_drift_after_manual_change() {
	let dir = TempDir::new().unwrap();

	create_manifest(dir.path(), "drift", "main", &[("core", "schema/core")]);

	fs::create_dir_all(dir.path().join("schema/core")).unwrap();
	fs::write(
		dir.path().join("schema/core/tables.surql"),
		"DEFINE TABLE user SCHEMAFULL;\nDEFINE FIELD name ON user TYPE string;",
	)
	.unwrap();

	let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
		.await
		.unwrap();

	// Load the project first
	let load_result = server
		.load_manifest(Parameters(ManifestArgs {
			path: dir.path().to_string_lossy().to_string(),
		}))
		.await
		.unwrap();
	assert!(
		result_text(&load_result).contains("1 schema module(s)"),
		"load should succeed"
	);

	// Manually add an extra table to the playground (drift)
	let run_result = server
		.run_query(Parameters(ExecArgs {
			query: "DEFINE TABLE extra_drift SCHEMAFULL".into(),
		}))
		.await
		.unwrap();
	assert!(run_result.is_error != Some(true), "query should succeed");

	// Get playground schema (has extra_drift)
	let schema_result = server.schema().await.unwrap();
	let schema_text = result_text(&schema_result);
	let json_start = schema_text.find('{').expect("schema has JSON");
	let json_end = schema_text.rfind('}').expect("schema has JSON") + 1;
	let playground_json = &schema_text[json_start..json_end];

	// Compare against expected (no extra_drift -- just user)
	let expected = serde_json::json!({
		"tables": {
			"user": "DEFINE TABLE user TYPE NORMAL SCHEMAFULL"
		},
		"functions": {}
	});

	let compare_result = server
		.compare(Parameters(CompareArgs {
			expected_json: expected.to_string(),
		}))
		.await
		.unwrap();
	let compare_text = result_text(&compare_result);
	assert!(
		compare_text.contains("extra_drift") && compare_text.contains("extra"),
		"expected extra_drift detected as extra table: {compare_text}"
	);

	// Also verify that playground_json contains the extra table
	let parsed: serde_json::Value = serde_json::from_str(playground_json).unwrap();
	let tables = parsed.get("tables").and_then(|t| t.as_object()).unwrap();
	assert!(
		tables.contains_key("extra_drift"),
		"playground schema should contain extra_drift table"
	);
}

#[tokio::test]
async fn should_handle_project_with_only_schema() {
	let dir = TempDir::new().unwrap();

	create_manifest(
		dir.path(),
		"schema_only",
		"main",
		&[("auth", "schema/auth")],
	);

	fs::create_dir_all(dir.path().join("schema/auth")).unwrap();
	fs::write(
		dir.path().join("schema/auth/tables.surql"),
		"DEFINE TABLE session SCHEMAFULL;\n\
		 DEFINE FIELD token ON session TYPE string;\n\
		 DEFINE FIELD user_id ON session TYPE record<user>;",
	)
	.unwrap();

	// No migrations/ directory at all

	let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
		.await
		.unwrap();

	// Load manifest with schema only
	let load_result = server
		.load_manifest(Parameters(ManifestArgs {
			path: dir.path().to_string_lossy().to_string(),
		}))
		.await
		.unwrap();
	let load_text = result_text(&load_result);
	assert!(
		load_text.contains("1 schema module(s)"),
		"expected 1 schema module: {load_text}"
	);
	assert!(
		load_text.contains("0 migration(s)"),
		"expected 0 migrations: {load_text}"
	);

	// Verify still works with schema-only project
	let verify_result = server
		.verify(Parameters(VerifyArgs {
			verify_only: None,
			path: dir.path().to_string_lossy().to_string(),
		}))
		.await
		.unwrap();
	let verify_text = result_text(&verify_result);
	assert!(
		verify_text.contains("Schema matches"),
		"schema-only project should match: {verify_text}"
	);
	assert!(
		verify_text.contains("0 migration(s)"),
		"expected 0 migrations in verify: {verify_text}"
	);

	// Check the table exists
	let describe_result = server
		.describe(Parameters(DescribeArgs {
			table: "session".into(),
		}))
		.await
		.unwrap();
	let describe_text = result_text(&describe_result);
	assert!(
		describe_text.contains("token"),
		"expected token field: {describe_text}"
	);
}

#[tokio::test]
async fn should_handle_project_with_only_migrations() {
	let dir = TempDir::new().unwrap();

	// No modules in manifest
	create_manifest(dir.path(), "mig_only", "main", &[]);

	fs::create_dir_all(dir.path().join("migrations")).unwrap();
	fs::write(
		dir.path().join("migrations/v001_init.surql"),
		"DEFINE TABLE product SCHEMAFULL;\n\
		 DEFINE FIELD name ON product TYPE string;\n\
		 DEFINE FIELD price ON product TYPE float;",
	)
	.unwrap();
	fs::write(
		dir.path().join("migrations/v002_index.surql"),
		"DEFINE INDEX idx_product_name ON product FIELDS name;",
	)
	.unwrap();

	let server = SurqlMcp::with_workspace_root(dir.path().to_path_buf())
		.await
		.unwrap();

	// Load manifest with migrations only
	let load_result = server
		.load_manifest(Parameters(ManifestArgs {
			path: dir.path().to_string_lossy().to_string(),
		}))
		.await
		.unwrap();
	let load_text = result_text(&load_result);
	assert!(
		load_text.contains("0 schema module(s)"),
		"expected 0 schema modules: {load_text}"
	);
	assert!(
		load_text.contains("2 migration(s)"),
		"expected 2 migrations: {load_text}"
	);

	// Verify works with migrations-only project
	let verify_result = server
		.verify(Parameters(VerifyArgs {
			verify_only: None,
			path: dir.path().to_string_lossy().to_string(),
		}))
		.await
		.unwrap();
	let verify_text = result_text(&verify_result);
	assert!(
		verify_text.contains("Schema matches"),
		"migrations-only project should match: {verify_text}"
	);
	assert!(
		verify_text.contains("0 module(s)"),
		"expected 0 modules in verify: {verify_text}"
	);
	assert!(
		verify_text.contains("2 migration(s)"),
		"expected 2 migrations in verify: {verify_text}"
	);

	// Check the table and index exist
	let describe_result = server
		.describe(Parameters(DescribeArgs {
			table: "product".into(),
		}))
		.await
		.unwrap();
	let describe_text = result_text(&describe_result);
	assert!(
		describe_text.contains("name"),
		"expected name field: {describe_text}"
	);
	assert!(
		describe_text.contains("price"),
		"expected price field: {describe_text}"
	);
}
