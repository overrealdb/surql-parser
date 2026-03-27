//! Tests for extract_params — parameter extraction from SurrealQL queries.

use surql_parser::extract_params;

#[test]
fn no_params() {
	let params = extract_params("SELECT * FROM user").unwrap();
	assert!(params.is_empty());
}

#[test]
fn single_param() {
	let params = extract_params("SELECT * FROM user WHERE age > $min").unwrap();
	assert_eq!(params, vec!["min"]);
}

#[test]
fn multiple_params() {
	let params =
		extract_params("SELECT * FROM user WHERE age > $min AND name = $name LIMIT $lim").unwrap();
	assert_eq!(params, vec!["lim", "min", "name"]); // sorted
}

#[test]
fn duplicate_params_deduplicated() {
	let params = extract_params("SELECT * FROM user WHERE $x > 1 AND $x < 10 OR $y = $x").unwrap();
	assert_eq!(params, vec!["x", "y"]);
}

#[test]
fn params_in_create() {
	let params = extract_params("CREATE user SET name = $name, age = $age").unwrap();
	assert_eq!(params, vec!["age", "name"]);
}

#[test]
fn param_in_string_ignored() {
	let params = extract_params("SELECT * FROM user WHERE name = '$not_a_param'").unwrap();
	assert!(params.is_empty());
}

#[test]
fn param_in_double_quoted_string_ignored() {
	let params = extract_params(r#"SELECT * FROM user WHERE name = "$not_a_param""#).unwrap();
	assert!(params.is_empty());
}

#[test]
fn param_in_comment_ignored() {
	let params =
		extract_params("SELECT * FROM user WHERE age > $min -- $commented_out\n AND name = $name")
			.unwrap();
	assert_eq!(params, vec!["min", "name"]);
}

#[test]
fn param_in_block_comment_ignored() {
	let params =
		extract_params("SELECT * FROM user WHERE age > $min /* $hidden */ AND name = $name")
			.unwrap();
	assert_eq!(params, vec!["min", "name"]);
}

#[test]
fn let_binding() {
	let params = extract_params("LET $x = 42; SELECT * FROM user WHERE age > $x").unwrap();
	assert_eq!(params, vec!["x"]);
}

#[test]
fn invalid_sql_returns_error() {
	assert!(extract_params("SELEC * FORM user").is_err());
}
