use surql_macros::surql_check;

fn main() {
	let _ = surql_check!("SELECT * FROM user WHERE name = 'Alice");
}
