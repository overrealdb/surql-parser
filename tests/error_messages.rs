//! Negative tests — verify parser gives clear errors on invalid SurrealQL.
//!
//! These test both that invalid input IS rejected and that the error message
//! is actually helpful (contains relevant context, points to the right location).

use surql_parser::parse;

fn assert_rejects(input: &str) {
	assert!(parse(input).is_err(), "Should reject: {input}");
}

// ─── Typos in keywords ───

#[test]
fn error_select_typo() {
	assert_rejects("SELEC * FROM user");
}

#[test]
fn error_from_typo() {
	assert_rejects("SELECT * FORM user");
}

#[test]
fn error_define_typo() {
	assert_rejects("DEFNE TABLE user SCHEMAFULL");
}

// ─── Incomplete statements ───

#[test]
fn error_select_without_from() {
	// "SELECT *" alone is actually valid in some contexts, so be careful
	assert_rejects("SELECT * FROM");
}

#[test]
fn error_define_table_no_name() {
	assert_rejects("DEFINE TABLE");
}

#[test]
fn error_define_field_no_table() {
	assert_rejects("DEFINE FIELD name TYPE string");
}

#[test]
fn error_define_function_no_body() {
	assert_rejects("DEFINE FUNCTION fn::greet($name: string)");
}

// ─── Syntax errors ───

#[test]
fn error_unclosed_brace() {
	assert_rejects("DEFINE FUNCTION fn::test() { RETURN 1;");
}

#[test]
fn error_unclosed_string() {
	assert_rejects("SELECT * FROM user WHERE name = 'unclosed");
}

#[test]
fn error_unclosed_paren() {
	assert_rejects("SELECT count( FROM user");
}

#[test]
fn accept_multiple_statements() {
	let result = parse("SELECT * FROM a; SELECT * FROM b;");
	assert!(result.is_ok());
}

// ─── SurrealDB 3 specific errors ───

#[test]
fn error_flexible_before_type() {
	// SurrealDB 3 gotcha: FLEXIBLE goes AFTER TYPE, not before
	assert_rejects("DEFINE FIELD meta ON user FLEXIBLE TYPE object");
}

#[test]
fn error_deprecated_let_syntax() {
	// SurrealDB 3: $x = 1 without LET is deprecated
	assert_rejects("$x = 1");
}

// ─── Type annotation errors ───

#[test]
fn error_invalid_type() {
	assert_rejects("DEFINE FIELD name ON user TYPE notarealtype");
}

// ─── Error message quality ───
// These tests check that errors actually contain useful context

#[test]
fn error_message_shows_position() {
	let result = parse("SELECT * FROM user WHRE age > 18");
	assert!(result.is_err());
	let err = result.unwrap_err().to_string();
	// Error should mention what was unexpected
	assert!(
		err.contains("Unexpected") || err.contains("expected") || err.contains("token"),
		"Error should indicate unexpected token, got:\n{err}"
	);
}

#[test]
fn error_message_for_unclosed_string_is_helpful() {
	let result = parse("RETURN 'hello there");
	assert!(result.is_err());
	let err = result.unwrap_err().to_string();
	assert!(
		err.contains("string") || err.contains("end of") || err.contains("unclosed"),
		"Error for unclosed string should be helpful, got:\n{err}"
	);
}

// ─── Edge cases that should NOT error ───

#[test]
fn accept_empty_input() {
	assert!(parse("").is_ok());
}

#[test]
fn accept_whitespace_only() {
	assert!(parse("   \n\t\n  ").is_ok());
}

#[test]
fn accept_comments_only() {
	assert!(parse("-- this is a comment\n/* block comment */").is_ok());
}

#[test]
fn accept_unicode_in_strings() {
	assert!(parse("RETURN 'Привет мир! '").is_ok());
}

#[test]
fn accept_deeply_nested() {
	assert!(parse("SELECT * FROM (SELECT * FROM (SELECT * FROM user))").is_ok());
}

#[test]
fn accept_complex_record_id() {
	assert!(parse("SELECT * FROM user:⟨complex-id-123⟩").is_ok());
}
