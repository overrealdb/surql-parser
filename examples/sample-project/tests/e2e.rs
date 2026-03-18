//! End-to-end tests: build.rs codegen + proc macros in a real project.

use std::path::Path;
use surql_sample_project::*;

// ─── Generated constants (from build.rs → generate_typed_functions) ───

#[test]
fn generated_constant_get_user() {
	assert_eq!(FN_GET_USER, "fn::get_user");
}

#[test]
fn generated_constant_create_post() {
	assert_eq!(FN_CREATE_POST, "fn::create_post");
}

#[test]
fn generated_constant_user_list() {
	assert_eq!(FN_USER_LIST, "fn::user::list");
}

#[test]
fn generated_constant_user_by_age() {
	assert_eq!(FN_USER_BY_AGE, "fn::user::by_age");
}

// ─── surql_check! validated queries ───

#[test]
fn surql_check_simple_select() {
	assert_eq!(QUERY_ALL_USERS, "SELECT * FROM user");
}

#[test]
fn surql_check_where_clause() {
	assert_eq!(QUERY_BY_AGE, "SELECT * FROM user WHERE age > 18");
}

#[test]
fn surql_check_projection() {
	assert!(QUERY_USER_POSTS.contains("author_name"));
}

#[test]
fn surql_check_multi_statement() {
	assert!(QUERY_MULTI.contains("SELECT name FROM user"));
	assert!(QUERY_MULTI.contains("SELECT title"));
}

#[test]
fn surql_check_define_statement() {
	assert_eq!(QUERY_DEFINE, "DEFINE TABLE audit SCHEMALESS");
}

// ─── #[surql_function] wrappers ───

#[test]
fn surql_function_get_user() {
	let call = get_user_call("user:abc123");
	assert!(call.starts_with("fn::get_user("));
	assert!(call.contains("user:abc123"));
}

#[test]
fn surql_function_create_post() {
	let call = create_post_call("Hello", "World", "user:1");
	assert!(call.starts_with("fn::create_post("));
	assert!(call.contains("Hello"));
	assert!(call.contains("World"));
	assert!(call.contains("user:1"));
}

#[test]
fn surql_function_list_users() {
	assert_eq!(list_users_call(), "fn::user::list()");
}

#[test]
fn surql_function_users_by_age() {
	let call = users_by_age_call(18, 65);
	assert_eq!(call, "fn::user::by_age(18, 65)");
}

// ─── Cross-layer: generated constant matches function wrapper ───

#[test]
fn constant_matches_wrapper_get_user() {
	let call = get_user_call("test");
	assert!(call.starts_with(FN_GET_USER));
}

#[test]
fn constant_matches_wrapper_list_users() {
	let call = list_users_call();
	assert!(call.starts_with(FN_USER_LIST));
}

// ─── Negative: build helper runtime failures ───

fn create_temp_schema(dir: &Path, filename: &str, content: &str) {
	std::fs::create_dir_all(dir).unwrap();
	std::fs::write(dir.join(filename), content).unwrap();
}

#[test]
#[should_panic(expected = "SurrealQL validation failed")]
fn build_validate_rejects_invalid_schema() {
	let dir = tempfile::tempdir().unwrap();
	create_temp_schema(dir.path(), "bad.surql", "SELCT * FORM user;");
	surql_parser::build::validate_schema(dir.path());
}

#[test]
#[should_panic(expected = "SurrealQL validation failed")]
fn build_validate_rejects_unclosed_string() {
	let dir = tempfile::tempdir().unwrap();
	create_temp_schema(
		dir.path(),
		"bad.surql",
		"SELECT * FROM user WHERE name = 'Alice",
	);
	surql_parser::build::validate_schema(dir.path());
}

#[test]
#[should_panic(expected = "2 error")]
fn build_validate_collects_all_errors() {
	let dir = tempfile::tempdir().unwrap();
	create_temp_schema(dir.path(), "a.surql", "SELEC broken");
	create_temp_schema(dir.path(), "b.surql", "ALSO broken syntax!!!");
	create_temp_schema(dir.path(), "c.surql", "SELECT * FROM user;\n"); // this one is OK
	surql_parser::build::validate_schema(dir.path());
}

#[test]
#[should_panic(expected = "Failed to parse schema files")]
fn build_generate_rejects_invalid_schema() {
	let schema_dir = tempfile::tempdir().unwrap();
	create_temp_schema(schema_dir.path(), "bad.surql", "NOT VALID SQL AT ALL");
	let out = tempfile::tempdir().unwrap();
	surql_parser::build::generate_typed_functions(schema_dir.path(), out.path().join("out.rs"));
}

// ─── Negative: parser rejects invalid SQL at runtime ───

#[test]
fn parser_rejects_invalid_sql() {
	assert!(surql_parser::parse("SELCT * FORM user").is_err());
}

#[test]
fn parser_rejects_unclosed_string() {
	assert!(surql_parser::parse("SELECT * FROM user WHERE name = 'oops").is_err());
}

#[test]
fn parser_rejects_unclosed_paren() {
	assert!(surql_parser::parse("SELECT * FROM (SELECT * FROM user").is_err());
}

#[test]
fn parser_rejects_incomplete_define() {
	assert!(surql_parser::parse("DEFINE TABLE").is_err());
}

// ─── Edge cases: function wrappers with boundary input ───

#[test]
fn function_wrapper_with_empty_string() {
	let call = get_user_call("");
	assert_eq!(call, "fn::get_user('')");
}

#[test]
fn function_wrapper_with_special_chars() {
	let call = get_user_call("user's \"name\" & <value>");
	assert!(call.starts_with("fn::get_user("));
	assert!(call.contains("user's \"name\" & <value>"));
}

#[test]
fn function_wrapper_with_unicode() {
	let call = get_user_call("用户名");
	assert!(call.contains("用户名"));
}

#[test]
fn function_wrapper_zero_range() {
	let call = users_by_age_call(0, 0);
	assert_eq!(call, "fn::user::by_age(0, 0)");
}

#[test]
fn function_wrapper_negative_values() {
	let call = users_by_age_call(-10, -1);
	assert_eq!(call, "fn::user::by_age(-10, -1)");
}

#[test]
fn function_wrapper_max_values() {
	let call = users_by_age_call(i64::MIN, i64::MAX);
	assert!(call.contains(&i64::MIN.to_string()));
	assert!(call.contains(&i64::MAX.to_string()));
}
