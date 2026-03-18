use surql_macros::surql_check;

fn main() {
	let q = surql_check!("SELECT * FROM user");
	assert_eq!(q, "SELECT * FROM user");
}
