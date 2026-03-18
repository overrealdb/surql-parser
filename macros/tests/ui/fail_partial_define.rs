use surql_macros::surql_check;

fn main() {
	// Incomplete DEFINE statement
	let _ = surql_check!("DEFINE TABLE");
}
