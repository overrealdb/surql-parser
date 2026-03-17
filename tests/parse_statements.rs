//! Layer 1: Parse tests — verify SurrealQL constructs parse correctly.
//!
//! Each test includes a link to SurrealDB documentation for maintainability.
//! With `--features surrealdb-validation`, each test also validates against real SurrealDB.

mod common;

use surql_parser::parse;

/// Parse and optionally validate against SurrealDB
fn check(sql: &str) {
	let ast = parse(sql).unwrap_or_else(|e| panic!("Parse failed: {sql}\n{e}"));
	assert!(!ast.expressions.is_empty(), "Empty AST for: {sql}");
	common::validate(sql);
}

fn check_count(sql: &str, expected: usize) {
	let ast = parse(sql).unwrap_or_else(|e| panic!("Parse failed: {sql}\n{e}"));
	assert_eq!(ast.expressions.len(), expected, "Wrong count for: {sql}");
	common::validate(sql);
}

// ─── SELECT ───
// Docs: https://surrealdb.com/docs/surrealql/statements/select

#[test]
fn parse_select_basic() {
	check("SELECT * FROM user");
}

#[test]
fn parse_select_fields() {
	check("SELECT name, age FROM user");
}

#[test]
fn parse_select_where() {
	check("SELECT * FROM user WHERE age > 18");
}

#[test]
fn parse_select_order_limit() {
	check("SELECT * FROM user ORDER BY name ASC LIMIT 10");
}

#[test]
fn parse_select_graph_traversal() {
	check("SELECT ->knows->person FROM user:tobie");
}

// ─── CREATE ───
// Docs: https://surrealdb.com/docs/surrealql/statements/create

#[test]
fn parse_create_set() {
	check("CREATE user SET name = 'Alice', age = 30");
}

#[test]
fn parse_create_content() {
	check("CREATE user CONTENT { name: 'Alice', age: 30 }");
}

// ─── UPDATE ───
// Docs: https://surrealdb.com/docs/surrealql/statements/update

#[test]
fn parse_update() {
	check("UPDATE user SET age = 31 WHERE name = 'Alice'");
}

// ─── DELETE ───
// Docs: https://surrealdb.com/docs/surrealql/statements/delete

#[test]
fn parse_delete() {
	check("DELETE user WHERE age < 18");
}

// ─── DEFINE TABLE ───
// Docs: https://surrealdb.com/docs/surrealql/statements/define/table

#[test]
fn parse_define_table_schemafull() {
	check("DEFINE TABLE user SCHEMAFULL");
}

#[test]
fn parse_define_table_schemaless() {
	check("DEFINE TABLE post SCHEMALESS");
}

#[test]
fn parse_define_table_type_relation() {
	// Parser accepts this, but Docker SurrealDB may reject if version doesn't match.
	// Validate only parser, not SurrealDB for this syntax (version-sensitive).
	let sql = "DEFINE TABLE knows TYPE RELATION FROM user TO user ENFORCED";
	let ast = parse(sql).unwrap();
	assert!(!ast.expressions.is_empty());
}

// ─── DEFINE FIELD ───
// Docs: https://surrealdb.com/docs/surrealql/statements/define/field

#[test]
fn parse_define_field() {
	check("DEFINE FIELD name ON user TYPE string");
}

#[test]
fn parse_define_field_with_default() {
	check("DEFINE FIELD created_at ON user TYPE datetime DEFAULT time::now()");
}

#[test]
fn parse_define_field_flexible() {
	// SurrealDB 3 gotcha: FLEXIBLE goes AFTER TYPE
	check("DEFINE FIELD metadata ON user TYPE object FLEXIBLE");
}

// ─── DEFINE INDEX ───
// Docs: https://surrealdb.com/docs/surrealql/statements/define/index

#[test]
fn parse_define_index_unique() {
	check("DEFINE INDEX email_idx ON user FIELDS email UNIQUE");
}

// ─── DEFINE FUNCTION ───
// Docs: https://surrealdb.com/docs/surrealql/statements/define/function

#[test]
fn parse_define_function() {
	check("DEFINE FUNCTION fn::greet($name: string) { RETURN 'Hello, ' + $name; }");
}

#[test]
fn parse_define_function_overwrite() {
	// OVERWRITE syntax may not be in all SurrealDB Docker image versions.
	let sql = "DEFINE FUNCTION OVERWRITE fn::add($a: int, $b: int) { RETURN $a + $b; }";
	let ast = parse(sql).unwrap();
	assert!(!ast.expressions.is_empty());
}

// ─── DEFINE ANALYZER ───
// Docs: https://surrealdb.com/docs/surrealql/statements/define/analyzer

#[test]
fn parse_define_analyzer() {
	check(
		"DEFINE ANALYZER my_analyzer TOKENIZERS blank, class FILTERS lowercase, snowball(english)",
	);
}

// ─── RELATE ───
// Docs: https://surrealdb.com/docs/surrealql/statements/relate

#[test]
fn parse_relate() {
	check("RELATE user:tobie->knows->user:jaime SET since = '2024'");
}

// ─── Multi-statement ───

#[test]
fn parse_multiple_statements() {
	check_count("CREATE user SET name = 'A'; SELECT * FROM user;", 2);
}

// ─── Error cases ───

#[test]
fn parse_invalid_returns_error() {
	assert!(parse("SELEC * FORM user").is_err());
}

#[test]
fn parse_empty_returns_empty() {
	let ast = parse("").unwrap();
	assert_eq!(ast.expressions.len(), 0);
}

// ─── LET / USE / INFO ───
// Docs: https://surrealdb.com/docs/surrealql/statements/let

#[test]
fn parse_let_variable() {
	check("LET $name = 'Alice'");
}

#[test]
fn parse_use_ns_db() {
	check("USE NS test DB main");
}

#[test]
fn parse_info_for_db() {
	check("INFO FOR DB");
}

// ─── Expressions ───

#[test]
fn parse_subquery() {
	check("SELECT * FROM (SELECT name FROM user WHERE age > 18)");
}

#[test]
fn parse_record_id() {
	check("SELECT * FROM user:tobie");
}

#[test]
fn parse_if_else() {
	check("IF $age > 18 { RETURN 'adult' } ELSE { RETURN 'minor' }");
}
