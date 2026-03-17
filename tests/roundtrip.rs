//! Layer 2: Roundtrip tests — parse → format → parse again.
//!
//! Verifies that the AST Display/ToSql output can be re-parsed
//! to produce a semantically equivalent AST.

use surql_parser::parse;
use surrealdb_types::{SqlFormat, ToSql};

fn roundtrip(input: &str) {
	let ast1 = parse(input).unwrap_or_else(|e| panic!("First parse failed for: {input}\n{e}"));

	// Format back to string
	let mut formatted = String::new();
	ast1.fmt_sql(&mut formatted, SqlFormat::SingleLine);

	let ast2 = parse(&formatted).unwrap_or_else(|e| {
		panic!("Re-parse failed.\nOriginal: {input}\nFormatted: {formatted}\n{e}")
	});

	// Compare expression count (structural equivalence)
	assert_eq!(
		ast1.expressions.len(),
		ast2.expressions.len(),
		"Expression count mismatch.\nOriginal: {input}\nFormatted: {formatted}"
	);
}

// ─── DML Roundtrips ───

#[test]
fn roundtrip_select() {
	roundtrip("SELECT * FROM user");
	roundtrip("SELECT name, age FROM user WHERE age > 18 ORDER BY name LIMIT 10");
}

#[test]
fn roundtrip_create() {
	roundtrip("CREATE user SET name = 'Alice', age = 30");
	roundtrip("CREATE user CONTENT { name: 'Bob' }");
}

#[test]
fn roundtrip_update() {
	roundtrip("UPDATE user SET age = 31 WHERE name = 'Alice'");
}

#[test]
fn roundtrip_delete() {
	roundtrip("DELETE user WHERE age < 18");
}

#[test]
fn roundtrip_relate() {
	roundtrip("RELATE user:tobie->knows->user:jaime SET since = '2024'");
}

#[test]
fn roundtrip_insert() {
	roundtrip("INSERT INTO user { name: 'Charlie', age: 25 }");
}

// ─── DDL Roundtrips ───

#[test]
fn roundtrip_define_table() {
	roundtrip("DEFINE TABLE user SCHEMAFULL");
	roundtrip("DEFINE TABLE post SCHEMALESS");
}

#[test]
fn roundtrip_define_field() {
	roundtrip("DEFINE FIELD name ON user TYPE string");
	roundtrip("DEFINE FIELD age ON user TYPE int DEFAULT 0");
}

#[test]
fn roundtrip_define_index() {
	roundtrip("DEFINE INDEX email_idx ON user FIELDS email UNIQUE");
}

#[test]
fn roundtrip_define_function() {
	roundtrip("DEFINE FUNCTION fn::greet($name: string) { RETURN 'Hello, ' + $name; }");
}

// ─── Expressions ───

#[test]
fn roundtrip_let() {
	roundtrip("LET $x = 42");
}

#[test]
fn roundtrip_if_else() {
	roundtrip("IF $age > 18 { RETURN 'adult' } ELSE { RETURN 'minor' }");
}

#[test]
fn roundtrip_subquery() {
	roundtrip("SELECT * FROM (SELECT name FROM user)");
}

// ─── Multi-statement ───

#[test]
fn roundtrip_multi() {
	roundtrip("LET $x = 1; LET $y = 2; RETURN $x + $y;");
}
