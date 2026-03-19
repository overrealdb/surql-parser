use surql_macros::surql_query;

fn main() {
	// Extra parameter `city` not in query
	let _sql = surql_query!("SELECT * FROM user WHERE age > $min", min, city);
}
