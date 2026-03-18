use surql_macros::surql_function;

#[surql_function("fn::greet")]
fn greet(name: &str) -> String {
	format!("fn::greet('{name}')")
}

#[surql_function("fn::add")]
fn add(a: i64, b: i64) -> String {
	format!("fn::add({a}, {b})")
}

fn main() {
	let _ = greet("world");
	let _ = add(1, 2);
}
