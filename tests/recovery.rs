//! Tests for parse_with_recovery — error-recovering parser.

use surql_parser::parse_with_recovery;

#[test]
fn all_valid_returns_all_statements() {
	let (stmts, diags) = parse_with_recovery("SELECT * FROM user; SELECT * FROM post");
	assert_eq!(stmts.len(), 2);
	assert!(diags.is_empty());
}

#[test]
fn single_error_still_parses_others() {
	let (stmts, diags) =
		parse_with_recovery("SELECT * FROM user; SELEC broken; DEFINE TABLE post SCHEMAFULL");
	assert_eq!(stmts.len(), 2, "expected 2 valid statements");
	assert_eq!(diags.len(), 1, "expected 1 diagnostic");
}

#[test]
fn all_errors() {
	let (stmts, diags) = parse_with_recovery("SELEC broken; UPDAET also broken");
	assert!(stmts.is_empty());
	assert_eq!(diags.len(), 2);
}

#[test]
fn empty_input() {
	let (stmts, diags) = parse_with_recovery("");
	assert!(stmts.is_empty());
	assert!(diags.is_empty());
}

#[test]
fn whitespace_only() {
	let (stmts, diags) = parse_with_recovery("   \n\t  ");
	assert!(stmts.is_empty());
	assert!(diags.is_empty());
}

#[test]
fn single_valid_no_semicolon() {
	let (stmts, diags) = parse_with_recovery("SELECT * FROM user");
	assert_eq!(stmts.len(), 1);
	assert!(diags.is_empty());
}

#[test]
fn single_error_no_semicolon() {
	let (stmts, diags) = parse_with_recovery("SELEC broken");
	assert!(stmts.is_empty());
	assert_eq!(diags.len(), 1);
}

#[test]
fn trailing_semicolons() {
	let (stmts, diags) = parse_with_recovery("SELECT * FROM user;;;");
	assert_eq!(stmts.len(), 1);
	assert!(diags.is_empty());
}

#[test]
fn define_table_survives_broken_neighbor() {
	let (stmts, diags) = parse_with_recovery(
		"DEFINE TABLE user SCHEMAFULL; SELEC broken; DEFINE FIELD name ON user TYPE string",
	);
	assert_eq!(stmts.len(), 2);
	assert_eq!(diags.len(), 1);
}

#[test]
fn error_diagnostic_has_correct_line() {
	let (_, diags) = parse_with_recovery("SELECT * FROM user;\nSELEC broken");
	assert_eq!(diags.len(), 1);
	// Error is on line 2 (1-indexed)
	assert_eq!(diags[0].line, 2);
}

#[test]
fn error_diagnostic_has_message() {
	let (_, diags) = parse_with_recovery("SELEC broken");
	assert!(!diags[0].message.is_empty());
}

#[test]
fn complex_valid_with_one_error() {
	let source = "\
DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD name ON user TYPE string;
DEFINE FIELD age ON user TYPE int;
SELEC broken query here;
DEFINE FUNCTION fn::greet($name: string) { RETURN 'Hello, ' + $name; };
SELECT * FROM user WHERE age > 18;
";
	let (stmts, diags) = parse_with_recovery(source);
	assert_eq!(stmts.len(), 5, "expected 5 valid statements");
	assert_eq!(diags.len(), 1, "expected 1 error");
}

#[test]
fn multiline_statement_parsed_correctly() {
	let source = "\
SELECT
    name,
    age
FROM
    user
WHERE
    age > 18;
SELECT * FROM post";
	let (stmts, diags) = parse_with_recovery(source);
	assert_eq!(stmts.len(), 2);
	assert!(diags.is_empty());
}

#[test]
fn comments_preserved() {
	let (stmts, diags) = parse_with_recovery("-- comment\nSELECT * FROM user");
	assert_eq!(stmts.len(), 1);
	assert!(diags.is_empty());
}
