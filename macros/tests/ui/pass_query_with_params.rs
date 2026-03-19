use surql_macros::surql_query;

fn main() {
	let sql = surql_query!(
		"SELECT * FROM user WHERE age > $min AND name = $name",
		min,
		name
	);
	assert_eq!(
		sql,
		"SELECT * FROM user WHERE age > $min AND name = $name"
	);
}
