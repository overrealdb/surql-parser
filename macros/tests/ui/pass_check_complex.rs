use surql_macros::surql_check;

fn main() {
	// Multi-statement
	let _ = surql_check!("SELECT * FROM user; SELECT * FROM post");

	// WHERE clause
	let _ = surql_check!("SELECT name, age FROM user WHERE age > 18 ORDER BY name LIMIT 10");

	// Subquery
	let _ = surql_check!("SELECT * FROM user WHERE id IN (SELECT author FROM post)");

	// DEFINE
	let _ = surql_check!("DEFINE TABLE user SCHEMAFULL");
	let _ = surql_check!("DEFINE FIELD name ON user TYPE string");
	let _ = surql_check!("DEFINE INDEX email_idx ON user FIELDS email UNIQUE");

	// CREATE / UPDATE / DELETE
	let _ = surql_check!("CREATE user SET name = 'Alice', age = 30");
	let _ = surql_check!("UPDATE user SET age = 31 WHERE name = 'Alice'");
	let _ = surql_check!("DELETE user WHERE age < 0");

	// Empty input is valid
	let _ = surql_check!("");
}
