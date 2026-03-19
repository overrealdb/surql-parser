//! Tests for parse_for_diagnostics — structured error positions.

use surql_parser::parse_for_diagnostics;

#[test]
fn valid_query_returns_ok() {
	assert!(parse_for_diagnostics("SELECT * FROM user").is_ok());
}

#[test]
fn invalid_query_returns_diagnostics() {
	let diags = parse_for_diagnostics("SELEC * FROM user").unwrap_err();
	assert!(!diags.is_empty());
	assert_eq!(diags[0].line, 1);
	assert!(diags[0].message.contains("Unexpected"));
}

#[test]
fn error_position_is_accurate() {
	let diags = parse_for_diagnostics("SELECT * FORM user").unwrap_err();
	assert!(!diags.is_empty());
	// "FORM" starts at column 10 (1-indexed)
	let d = &diags[0];
	assert_eq!(d.line, 1);
	assert!(d.column >= 10);
}

#[test]
fn multiline_error_position() {
	let diags = parse_for_diagnostics("SELECT * FROM user;\nSELEC * FROM post").unwrap_err();
	assert!(!diags.is_empty());
	let d = &diags[0];
	assert_eq!(d.line, 2);
}

#[test]
fn unclosed_string_error() {
	let diags = parse_for_diagnostics("SELECT * FROM user WHERE name = 'unclosed").unwrap_err();
	assert!(!diags.is_empty());
}

#[test]
fn multiple_valid_statements() {
	assert!(parse_for_diagnostics("SELECT * FROM user; SELECT * FROM post").is_ok());
}
