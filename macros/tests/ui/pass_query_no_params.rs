use surql_macros::surql_query;

fn main() {
	let sql = surql_query!("SELECT * FROM user");
	assert_eq!(sql, "SELECT * FROM user");
}
