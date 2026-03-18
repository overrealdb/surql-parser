use surql_macros::surql_check;

fn main() {
	// Missing FROM clause
	let _ = surql_check!("SELECT * WHERE age > 18");
}
