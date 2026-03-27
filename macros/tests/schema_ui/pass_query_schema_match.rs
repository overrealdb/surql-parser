use surql_macros::surql_query;

fn main() {
	// Typed params matching schema field types
	let _sql = surql_query!(
		"SELECT * FROM user WHERE age > $min AND name = $name",
		min: i64,
		name: String,
		schema = "surql_fixtures/"
	);

	// Without types — schema validation still passes (no type to check)
	let _sql2 = surql_query!(
		"SELECT * FROM user WHERE active = $flag",
		flag,
		schema = "surql_fixtures/"
	);

	// Equality with bool field
	let _sql3 = surql_query!(
		"SELECT * FROM user WHERE active = $is_active",
		is_active: bool,
		schema = "surql_fixtures/"
	);
}
