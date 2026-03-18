//! Property-based tests — parse never panics, roundtrip is stable.

use proptest::prelude::*;
use surql_parser::parse;
use surrealdb_types::{SqlFormat, ToSql};

// ─── Fuzz: parse(random) never panics ───

proptest! {
	#[test]
	fn parse_random_never_panics(input in "\\PC{0,256}") {
		let _ = parse(&input);
	}

	#[test]
	fn parse_ascii_never_panics(input in "[a-zA-Z0-9 ;,.*(){}\\[\\]'\"=<>!@#$%^&|/\\\\:_-]{0,128}") {
		let _ = parse(&input);
	}

	#[test]
	fn parse_kind_never_panics(input in "[a-zA-Z0-9<>, _|]{0,64}") {
		let _ = surql_parser::parse_kind(&input);
	}
}

// ─── Roundtrip: parse → format → parse is stable ───

/// Generate a safe identifier (backtick-escaped to avoid reserved keyword collisions).
fn safe_ident() -> impl Strategy<Value = String> {
	"[a-z]{1,8}".prop_map(|s| format!("`{s}`"))
}

fn surql_statement() -> impl Strategy<Value = String> {
	prop_oneof![
		Just("SELECT * FROM user".to_string()),
		Just("SELECT name FROM user WHERE age > 18".to_string()),
		Just("CREATE user SET name = 'Alice'".to_string()),
		Just("UPDATE user SET age = 31 WHERE name = 'Bob'".to_string()),
		Just("DELETE user WHERE active = false".to_string()),
		Just("DEFINE TABLE user SCHEMAFULL".to_string()),
		Just("DEFINE FIELD name ON user TYPE string".to_string()),
		Just("DEFINE INDEX idx ON user FIELDS email UNIQUE".to_string()),
		Just("LET $x = 42".to_string()),
		Just("INSERT INTO user { name: 'Charlie' }".to_string()),
		Just("RELATE user:a->knows->user:b".to_string()),
		// Parameterized variants
		(1..100i64).prop_map(|n| format!("SELECT * FROM user LIMIT {n}")),
		safe_ident().prop_map(|name| format!("SELECT * FROM {name}")),
		(safe_ident(), safe_ident())
			.prop_map(|(table, field)| { format!("DEFINE FIELD {field} ON {table} TYPE string") }),
	]
}

proptest! {
	#[test]
	fn roundtrip_stability(input in surql_statement()) {
		let ast1 = parse(&input).unwrap();

		let mut formatted = String::new();
		ast1.fmt_sql(&mut formatted, SqlFormat::SingleLine);

		let ast2 = parse(&formatted).unwrap();
		assert_eq!(ast1.expressions.len(), ast2.expressions.len());
	}
}
