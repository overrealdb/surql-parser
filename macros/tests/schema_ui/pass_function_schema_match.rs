use surql_macros::surql_function;

#[surql_function("fn::add", schema = "surql_fixtures/")]
fn add(a: i64, b: i64) -> String {
	format!("fn::add({a}, {b})")
}

#[surql_function("fn::greet", schema = "surql_fixtures/")]
fn greet(name: &str) -> String {
	format!("fn::greet('{name}')")
}

fn main() {
	let _ = add(1, 2);
	let _ = greet("world");
}
