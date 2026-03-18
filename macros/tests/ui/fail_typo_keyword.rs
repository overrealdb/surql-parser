use surql_macros::surql_check;

fn main() {
	// Typo: SELCT instead of SELECT
	let _ = surql_check!("SELCT name FROM user");
}
