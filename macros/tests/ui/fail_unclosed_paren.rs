use surql_macros::surql_check;

fn main() {
	// Unclosed parenthesis
	let _ = surql_check!("SELECT * FROM (SELECT * FROM user");
}
