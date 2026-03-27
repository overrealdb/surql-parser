use surql_macros::surql_query;

fn main() {
	// age is int in schema, but providing bool
	let _sql = surql_query!(
		"SELECT * FROM user WHERE age > $min",
		min: bool,
		schema = "surql_fixtures/"
	);
}
