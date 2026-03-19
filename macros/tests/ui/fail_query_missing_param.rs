use surql_macros::surql_query;

fn main() {
	// Missing $name parameter
	let _sql = surql_query!("SELECT * FROM user WHERE age > $min AND name = $name", min);
}
