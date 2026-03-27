use surql_macros::surql_function;

// Without schema = "..." the macro only validates the function name (no arity check).
// Schema validation is tested via examples/sample-project which has real .surql files.

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
