//! End-to-end tests: build.rs codegen + proc macros in a real project.

use std::path::Path;
use surql_sample_project::*;

// ─── Generated constants (from build.rs → generate_typed_functions) ───

#[test]
fn generated_constant_project_summary() {
	assert_eq!(FN_PROJECT_SUMMARY, "fn::project::summary");
}

#[test]
fn generated_constant_migration_apply() {
	assert_eq!(FN_MIGRATION_APPLY, "fn::migration::apply");
}

#[test]
fn generated_constant_agent_by_role() {
	assert_eq!(FN_AGENT_BY_ROLE, "fn::agent::by_role");
}

#[test]
fn generated_constant_sync_record() {
	assert_eq!(FN_SYNC_RECORD, "fn::sync::record");
}

// ─── surql_check! validated queries ───

#[test]
fn surql_check_all_agents() {
	assert!(QUERY_ALL_AGENTS.contains("FROM agent"));
}

#[test]
fn surql_check_projects() {
	assert_eq!(QUERY_PROJECTS, "SELECT * FROM project");
}

#[test]
fn surql_check_pending_migrations() {
	assert!(QUERY_PENDING.contains("status = 'pending'"));
}

// ─── surql_query! with parameters ───

#[test]
fn surql_query_agent_by_role() {
	assert!(QUERY_AGENT_BY_ROLE.contains("$role"));
}

#[test]
fn surql_query_migrations() {
	assert!(QUERY_MIGRATIONS.contains("$project"));
	assert!(QUERY_MIGRATIONS.contains("$min_version"));
}

// ─── #[surql_function] wrappers ───

#[test]
fn surql_function_project_summary() {
	let call = project_summary("project:surql_parser");
	assert!(call.starts_with("fn::project::summary("));
	assert!(call.contains("surql_parser"));
}

#[test]
fn surql_function_migration_apply() {
	let call = migration_apply("migration:v001", "agent:overseer");
	assert!(call.starts_with("fn::migration::apply("));
}

#[test]
fn surql_function_agent_by_role_call() {
	let call = agent_by_role("deployer");
	assert_eq!(call, "fn::agent::by_role('deployer')");
}

// ─── Cross-layer: generated constant matches function wrapper ───

#[test]
fn constant_matches_wrapper_project_summary() {
	let call = project_summary("test");
	assert!(call.starts_with(FN_PROJECT_SUMMARY));
}

// ─── Negative: build helper runtime failures ───

fn create_temp_schema(dir: &Path, filename: &str, content: &str) {
	std::fs::create_dir_all(dir).unwrap();
	std::fs::write(dir.join(filename), content).unwrap();
}

#[test]
fn build_validate_reports_invalid_schema() {
	let dir = tempfile::tempdir().unwrap();
	create_temp_schema(dir.path(), "bad.surql", "SELCT * FORM user;");
	let errors = surql_parser::build::validate_schema(dir.path());
	assert_eq!(errors, 1, "should report 1 error for invalid schema");
}

#[test]
fn build_validate_collects_all_errors() {
	let dir = tempfile::tempdir().unwrap();
	create_temp_schema(dir.path(), "a.surql", "SELEC broken");
	create_temp_schema(dir.path(), "b.surql", "ALSO broken syntax!!!");
	create_temp_schema(dir.path(), "c.surql", "SELECT * FROM agent;\n");
	let errors = surql_parser::build::validate_schema(dir.path());
	assert_eq!(errors, 2, "should report 2 errors, c.surql should be OK");
}

#[test]
fn build_generate_tolerates_invalid_schema() {
	let schema_dir = tempfile::tempdir().unwrap();
	create_temp_schema(schema_dir.path(), "bad.surql", "NOT VALID SQL AT ALL");
	create_temp_schema(
		schema_dir.path(),
		"good.surql",
		"DEFINE FUNCTION fn::test() -> string { RETURN 'ok'; };",
	);
	let out = tempfile::tempdir().unwrap();
	surql_parser::build::generate_typed_functions(schema_dir.path(), out.path().join("out.rs"));
	let content = std::fs::read_to_string(out.path().join("out.rs")).unwrap();
	assert!(content.contains("FN_TEST"));
}

// ─── Negative: parser rejects invalid SQL at runtime ───

#[test]
fn parser_rejects_invalid_sql() {
	assert!(surql_parser::parse("SELCT * FORM user").is_err());
}

#[test]
fn parser_rejects_unclosed_string() {
	assert!(surql_parser::parse("SELECT * FROM agent WHERE name = 'oops").is_err());
}
