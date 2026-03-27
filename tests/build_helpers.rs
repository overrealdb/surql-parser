#![cfg(feature = "build")]

use std::path::Path;

fn create_temp_schema(dir: &Path, filename: &str, content: &str) {
	std::fs::create_dir_all(dir).unwrap();
	std::fs::write(dir.join(filename), content).unwrap();
}

// ─── find_function_params tests ───

#[test]
fn should_find_single_param_function() {
	let dir = tempfile::tempdir().unwrap();
	create_temp_schema(
		dir.path(),
		"functions.surql",
		"DEFINE FUNCTION OVERWRITE fn::greet($name: string) -> string { RETURN 'Hello, ' + $name; };\n",
	);

	let params = surql_parser::find_function_params("fn::greet", dir.path())
		.unwrap()
		.expect("should find fn::greet");
	assert_eq!(params.len(), 1);
	assert_eq!(params[0].name, "name");
	assert_eq!(params[0].kind, "string");
}

#[test]
fn should_find_multi_param_function() {
	let dir = tempfile::tempdir().unwrap();
	create_temp_schema(
		dir.path(),
		"functions.surql",
		"DEFINE FUNCTION OVERWRITE fn::add($a: int, $b: int) -> int { RETURN $a + $b; };\n",
	);

	let params = surql_parser::find_function_params("fn::add", dir.path())
		.unwrap()
		.expect("should find fn::add");
	assert_eq!(params.len(), 2);
	assert_eq!(params[0].name, "a");
	assert_eq!(params[0].kind, "int");
	assert_eq!(params[1].name, "b");
	assert_eq!(params[1].kind, "int");
}

#[test]
fn should_find_zero_param_function() {
	let dir = tempfile::tempdir().unwrap();
	create_temp_schema(
		dir.path(),
		"functions.surql",
		"DEFINE FUNCTION OVERWRITE fn::ping() -> string { RETURN 'pong'; };\n",
	);

	let params = surql_parser::find_function_params("fn::ping", dir.path())
		.unwrap()
		.expect("should find fn::ping");
	assert_eq!(params.len(), 0);
}

#[test]
fn should_return_none_for_missing_function() {
	let dir = tempfile::tempdir().unwrap();
	create_temp_schema(
		dir.path(),
		"functions.surql",
		"DEFINE FUNCTION OVERWRITE fn::greet($name: string) -> string { RETURN 'Hello, ' + $name; };\n",
	);

	let result = surql_parser::find_function_params("fn::nonexistent", dir.path()).unwrap();
	assert!(result.is_none());
}

#[test]
fn should_find_function_across_multiple_surql_files() {
	let dir = tempfile::tempdir().unwrap();
	create_temp_schema(
		dir.path(),
		"a_functions.surql",
		"DEFINE FUNCTION OVERWRITE fn::greet($name: string) -> string { RETURN 'Hello, ' + $name; };\n",
	);
	create_temp_schema(
		dir.path(),
		"b_functions.surql",
		"DEFINE FUNCTION OVERWRITE fn::add($a: int, $b: int) -> int { RETURN $a + $b; };\n",
	);

	let params = surql_parser::find_function_params("fn::add", dir.path())
		.unwrap()
		.expect("should find fn::add in second file");
	assert_eq!(params.len(), 2);
}

#[test]
fn should_find_nested_namespace_function() {
	let dir = tempfile::tempdir().unwrap();
	create_temp_schema(
		dir.path(),
		"functions.surql",
		"DEFINE FUNCTION OVERWRITE fn::user::create($name: string, $email: string) -> object { RETURN {}; };\n",
	);

	let params = surql_parser::find_function_params("fn::user::create", dir.path())
		.unwrap()
		.expect("should find fn::user::create");
	assert_eq!(params.len(), 2);
	assert_eq!(params[0].name, "name");
	assert_eq!(params[1].name, "email");
}

#[test]
fn should_reject_name_without_fn_prefix() {
	let dir = tempfile::tempdir().unwrap();
	create_temp_schema(dir.path(), "functions.surql", "SELECT 1;\n");

	let result = surql_parser::find_function_params("greet", dir.path());
	assert!(result.is_err());
}

#[test]
fn should_handle_record_type_params() {
	let dir = tempfile::tempdir().unwrap();
	create_temp_schema(
		dir.path(),
		"functions.surql",
		"DEFINE FUNCTION OVERWRITE fn::summary($id: record<project>) -> object { RETURN {}; };\n",
	);

	let params = surql_parser::find_function_params("fn::summary", dir.path())
		.unwrap()
		.expect("should find fn::summary");
	assert_eq!(params.len(), 1);
	assert_eq!(params[0].name, "id");
	assert_eq!(params[0].kind, "record<project>");
}

#[test]
fn validate_schema_valid_files() {
	let dir = tempfile::tempdir().unwrap();
	create_temp_schema(
		dir.path(),
		"schema.surql",
		"DEFINE TABLE user SCHEMAFULL;\nDEFINE FIELD name ON user TYPE string;\n",
	);
	create_temp_schema(
		dir.path(),
		"functions.surql",
		"DEFINE FUNCTION fn::greet($name: string) { RETURN 'Hello, ' + $name; };\n",
	);

	// Should not panic
	surql_parser::build::validate_schema(dir.path());
}

#[test]
fn validate_schema_invalid_file() {
	let dir = tempfile::tempdir().unwrap();
	create_temp_schema(dir.path(), "bad.surql", "SELEC * FORM user;");

	let errors = surql_parser::build::validate_schema(dir.path());
	assert_eq!(errors, 1, "should report 1 invalid file");
}

#[test]
fn validate_schema_empty_dir() {
	let dir = tempfile::tempdir().unwrap();
	// Should not panic — no .surql files
	surql_parser::build::validate_schema(dir.path());
}

#[test]
fn generate_typed_functions_produces_constants() {
	let schema_dir = tempfile::tempdir().unwrap();
	create_temp_schema(
		schema_dir.path(),
		"functions.surql",
		"\
DEFINE FUNCTION fn::greet($name: string) { RETURN 'Hello, ' + $name; };
DEFINE FUNCTION fn::add($a: int, $b: int) -> int { RETURN $a + $b; };
",
	);

	let out_dir = tempfile::tempdir().unwrap();
	let out_file = out_dir.path().join("surql_functions.rs");

	surql_parser::build::generate_typed_functions(schema_dir.path(), &out_file);

	let generated = std::fs::read_to_string(&out_file).unwrap();

	// Check header
	assert!(generated.contains("Auto-generated by surql-parser"));

	// Check constants exist
	assert!(generated.contains("FN_GREET"));
	assert!(generated.contains("\"fn::greet\""));
	assert!(generated.contains("FN_ADD"));
	assert!(generated.contains("\"fn::add\""));

	// Check parameter docs
	assert!(generated.contains("$name: string"));
	assert!(generated.contains("$a: int, $b: int"));

	// Check return type doc
	assert!(generated.contains("Returns: `int`"));
}

#[test]
fn generate_typed_functions_empty_schema() {
	let schema_dir = tempfile::tempdir().unwrap();
	create_temp_schema(
		schema_dir.path(),
		"tables.surql",
		"DEFINE TABLE user SCHEMAFULL;\n",
	);

	let out_dir = tempfile::tempdir().unwrap();
	let out_file = out_dir.path().join("surql_functions.rs");

	surql_parser::build::generate_typed_functions(schema_dir.path(), &out_file);

	let generated = std::fs::read_to_string(&out_file).unwrap();
	// Should have header but no constants
	assert!(generated.contains("Auto-generated"));
	assert!(!generated.contains("pub const"));
}

#[test]
fn generate_typed_functions_nested_name() {
	let schema_dir = tempfile::tempdir().unwrap();
	create_temp_schema(
		schema_dir.path(),
		"functions.surql",
		"DEFINE FUNCTION fn::user::get($id: int) { RETURN $id; };\n",
	);

	let out_dir = tempfile::tempdir().unwrap();
	let out_file = out_dir.path().join("surql_functions.rs");

	surql_parser::build::generate_typed_functions(schema_dir.path(), &out_file);

	let generated = std::fs::read_to_string(&out_file).unwrap();
	assert!(generated.contains("FN_USER_GET"));
	assert!(generated.contains("\"fn::user::get\""));
}

// ─── validate_schema_or_fail tests ───

#[test]
#[should_panic(expected = "SurrealQL validation failed")]
fn validate_schema_or_fail_panics_on_invalid() {
	let dir = tempfile::tempdir().unwrap();
	create_temp_schema(dir.path(), "bad.surql", "SELEC * FORM user;");

	surql_parser::build::validate_schema_or_fail(dir.path());
}

#[test]
fn validate_schema_or_fail_succeeds_on_valid() {
	let dir = tempfile::tempdir().unwrap();
	create_temp_schema(
		dir.path(),
		"schema.surql",
		"DEFINE TABLE user SCHEMAFULL;\nDEFINE FIELD name ON user TYPE string;\n",
	);

	surql_parser::build::validate_schema_or_fail(dir.path());
}

// ─── Additional build helper tests ───

#[test]
fn validate_schema_skips_non_surql_files() {
	let dir = tempfile::tempdir().unwrap();
	// Valid .surql file
	create_temp_schema(dir.path(), "good.surql", "SELECT * FROM user;\n");
	// Non-.surql files with invalid content — should be ignored
	create_temp_schema(dir.path(), "notes.txt", "this is not sql at all!!!");
	create_temp_schema(dir.path(), "data.json", "{\"invalid\": \"sql\"}");
	create_temp_schema(dir.path(), "script.sh", "echo hello");

	// Should not panic — only .surql files are validated
	surql_parser::build::validate_schema(dir.path());
}

#[test]
fn validate_schema_nested_directories() {
	let dir = tempfile::tempdir().unwrap();
	let subdir = dir.path().join("migrations").join("v1");
	create_temp_schema(
		&subdir,
		"001_users.surql",
		"DEFINE TABLE user SCHEMAFULL;\n",
	);
	create_temp_schema(
		&subdir,
		"002_fields.surql",
		"DEFINE FIELD name ON user TYPE string;\n",
	);

	let subdir2 = dir.path().join("migrations").join("v2");
	create_temp_schema(
		&subdir2,
		"001_posts.surql",
		"DEFINE TABLE post SCHEMAFULL;\n",
	);

	surql_parser::build::validate_schema(dir.path());
}

#[test]
fn validate_schema_nested_invalid() {
	let dir = tempfile::tempdir().unwrap();
	let subdir = dir.path().join("deep").join("nested");
	create_temp_schema(subdir.as_path(), "bad.surql", "INVALID SYNTAX HERE");

	let errors = surql_parser::build::validate_schema(dir.path());
	assert_eq!(errors, 1, "should report 1 invalid nested file");
}

#[test]
fn validate_schema_reports_all_errors() {
	let dir = tempfile::tempdir().unwrap();
	create_temp_schema(dir.path(), "a_bad.surql", "SELEC * FROM user");
	create_temp_schema(dir.path(), "b_also_bad.surql", "CREAT user SET x = 1");
	create_temp_schema(
		dir.path(),
		"c_good.surql",
		"SELECT * FROM user;\n", // this one is fine
	);

	let errors = surql_parser::build::validate_schema(dir.path());
	assert_eq!(errors, 2, "should report exactly 2 invalid files");
}

#[test]
fn generate_typed_functions_from_multiple_files() {
	let schema_dir = tempfile::tempdir().unwrap();
	create_temp_schema(
		schema_dir.path(),
		"a_users.surql",
		"DEFINE FUNCTION fn::user::create($name: string) { RETURN CREATE user SET name = $name; };\n",
	);
	create_temp_schema(
		schema_dir.path(),
		"b_posts.surql",
		"DEFINE FUNCTION fn::post::list() -> array { RETURN SELECT * FROM post; };\n",
	);

	let out_dir = tempfile::tempdir().unwrap();
	let out_file = out_dir.path().join("surql_functions.rs");

	surql_parser::build::generate_typed_functions(schema_dir.path(), &out_file);

	let generated = std::fs::read_to_string(&out_file).unwrap();
	// Functions from both files should be present
	assert!(generated.contains("FN_USER_CREATE"));
	assert!(generated.contains("\"fn::user::create\""));
	assert!(generated.contains("FN_POST_LIST"));
	assert!(generated.contains("\"fn::post::list\""));
}

#[test]
fn generate_typed_functions_no_params_no_return() {
	let schema_dir = tempfile::tempdir().unwrap();
	create_temp_schema(
		schema_dir.path(),
		"functions.surql",
		"DEFINE FUNCTION fn::ping() { RETURN 'pong'; };\n",
	);

	let out_dir = tempfile::tempdir().unwrap();
	let out_file = out_dir.path().join("surql_functions.rs");

	surql_parser::build::generate_typed_functions(schema_dir.path(), &out_file);

	let generated = std::fs::read_to_string(&out_file).unwrap();
	assert!(generated.contains("FN_PING"));
	assert!(generated.contains("\"fn::ping\""));
	// No "Parameters:" line since no params
	assert!(!generated.contains("Parameters:"));
	// No "Returns:" line since no return type
	assert!(!generated.contains("Returns:"));
}

#[test]
fn generate_typed_functions_complex_types() {
	let schema_dir = tempfile::tempdir().unwrap();
	create_temp_schema(
		schema_dir.path(),
		"functions.surql",
		"\
DEFINE FUNCTION fn::find_related($record: record<user>, $depth: int) -> array {
	RETURN SELECT * FROM user;
};
",
	);

	let out_dir = tempfile::tempdir().unwrap();
	let out_file = out_dir.path().join("surql_functions.rs");

	surql_parser::build::generate_typed_functions(schema_dir.path(), &out_file);

	let generated = std::fs::read_to_string(&out_file).unwrap();
	assert!(generated.contains("FN_FIND_RELATED"));
	assert!(generated.contains("record<user>"));
	assert!(generated.contains("$depth: int"));
	assert!(generated.contains("Returns: `array`"));
}

#[test]
fn generate_typed_functions_output_is_valid_rust() {
	let schema_dir = tempfile::tempdir().unwrap();
	create_temp_schema(
		schema_dir.path(),
		"functions.surql",
		"\
DEFINE FUNCTION fn::greet($name: string) { RETURN 'Hello, ' + $name; };
DEFINE FUNCTION fn::add($a: int, $b: int) -> int { RETURN $a + $b; };
DEFINE FUNCTION fn::user::list() -> array { RETURN SELECT * FROM user; };
",
	);

	let out_dir = tempfile::tempdir().unwrap();
	let out_file = out_dir.path().join("surql_functions.rs");

	surql_parser::build::generate_typed_functions(schema_dir.path(), &out_file);

	let generated = std::fs::read_to_string(&out_file).unwrap();
	// Verify it's syntactically valid Rust by checking structure
	for line in generated.lines() {
		if line.starts_with("pub const") {
			assert!(line.contains(": &str = \"fn::"), "Bad const line: {line}");
			assert!(line.ends_with("\";"), "Const line not terminated: {line}");
		}
	}
}

// ─── docs generation tests ───

#[test]
fn should_generate_docs_with_comment_fields() {
	let schema = surql_parser::SchemaGraph::from_source(
		"\
DEFINE TABLE project SCHEMAFULL;
DEFINE FIELD name ON project TYPE string;
DEFINE FIELD ns ON project TYPE string COMMENT 'SurrealDB namespace';
DEFINE FIELD db ON project TYPE string COMMENT 'SurrealDB database';
DEFINE FIELD endpoint ON project TYPE none | string COMMENT 'ws://host:port';
DEFINE FIELD created_at ON project TYPE datetime DEFAULT time::now();
DEFINE INDEX project_name ON project FIELDS name UNIQUE;
",
	)
	.unwrap();

	let docs = schema.build_docs_markdown();

	assert!(docs.contains("# Schema Documentation"), "missing title");
	assert!(docs.contains("### project"), "missing table heading");
	assert!(docs.contains("SCHEMAFULL"), "missing schema type");
	assert!(
		docs.contains("SurrealDB namespace"),
		"missing COMMENT for ns field"
	);
	assert!(
		docs.contains("SurrealDB database"),
		"missing COMMENT for db field"
	);
	assert!(
		docs.contains("ws://host:port"),
		"missing COMMENT for endpoint field"
	);
	assert!(
		docs.contains("time::now()"),
		"missing default value for created_at"
	);
	assert!(
		docs.contains("`project_name` (UNIQUE)"),
		"missing unique index"
	);
}

#[test]
fn should_generate_docs_with_function_signatures() {
	let schema = surql_parser::SchemaGraph::from_source(
		"\
DEFINE FUNCTION fn::project::summary($id: record<project>) -> object { RETURN {}; };
DEFINE FUNCTION fn::agent::by_role($role: string) -> array { RETURN []; };
",
	)
	.unwrap();

	let docs = schema.build_docs_markdown();

	assert!(docs.contains("## Functions"), "missing functions section");
	assert!(
		docs.contains("### fn::project::summary"),
		"missing function heading"
	);
	assert!(
		docs.contains("$id: record<project>"),
		"missing parameter signature"
	);
	assert!(
		docs.contains("**Returns:** `object`"),
		"missing return type for project::summary"
	);
	assert!(
		docs.contains("### fn::agent::by_role"),
		"missing agent::by_role heading"
	);
	assert!(
		docs.contains("**Returns:** `array`"),
		"missing return type for agent::by_role"
	);
}

#[test]
fn should_generate_docs_from_sample_project() {
	let sample_dir =
		std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/sample-project/surql");
	if !sample_dir.exists() {
		return;
	}

	let schema = surql_parser::SchemaGraph::from_files(&sample_dir).unwrap();
	let docs = schema.build_docs_markdown();

	assert!(docs.contains("# Schema Documentation"), "missing title");
	assert!(docs.contains("### project"), "missing project table");
	assert!(
		docs.contains("SurrealDB namespace"),
		"missing COMMENT from project.ns"
	);
	assert!(
		docs.contains("fn::project::summary"),
		"missing function from functions.surql"
	);
}

#[test]
fn should_generate_empty_docs_for_empty_schema() {
	let schema = surql_parser::SchemaGraph::default();
	let docs = schema.build_docs_markdown();

	assert!(docs.contains("# Schema Documentation"), "missing title");
	assert!(
		!docs.contains("## Tables"),
		"should not have tables section"
	);
	assert!(
		!docs.contains("## Functions"),
		"should not have functions section"
	);
}

#[test]
fn should_include_table_comment_in_docs() {
	let schema = surql_parser::SchemaGraph::from_source(
		"DEFINE TABLE user SCHEMAFULL COMMENT 'Registered users of the platform';",
	)
	.unwrap();

	let docs = schema.build_docs_markdown();

	assert!(
		docs.contains("Registered users of the platform"),
		"missing table-level COMMENT"
	);
}

#[test]
fn should_include_events_in_docs() {
	let schema = surql_parser::SchemaGraph::from_source(
		"\
DEFINE TABLE user SCHEMAFULL;
DEFINE EVENT user_created ON user WHEN $event = 'CREATE' THEN {};
",
	)
	.unwrap();

	let docs = schema.build_docs_markdown();

	assert!(docs.contains("**Events:**"), "missing events section");
	assert!(docs.contains("`user_created`"), "missing event name");
}

#[test]
fn should_escape_pipe_in_type_column() {
	let schema = surql_parser::SchemaGraph::from_source(
		"\
DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD bio ON user TYPE none | string;
",
	)
	.unwrap();

	let docs = schema.build_docs_markdown();

	assert!(
		docs.contains("none \\| string"),
		"pipe in type should be escaped for markdown table: {docs}"
	);
}
