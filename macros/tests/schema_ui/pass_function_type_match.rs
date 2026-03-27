use surql_macros::surql_function;

#[surql_function("fn::add", schema = "surql_fixtures/")]
fn add(a: i64, b: i64) -> String {
	format!("fn::add({a}, {b})")
}

#[surql_function("fn::greet", schema = "surql_fixtures/")]
fn greet(name: &str) -> String {
	format!("fn::greet('{name}')")
}

#[surql_function("fn::get_user", schema = "surql_fixtures/")]
fn get_user(id: String) -> String {
	format!("fn::get_user('{id}')")
}

#[surql_function("fn::toggle", schema = "surql_fixtures/")]
fn toggle(flag: bool) -> String {
	format!("fn::toggle({flag})")
}

#[surql_function("fn::scale", schema = "surql_fixtures/")]
fn scale(value: f64, factor: f64) -> String {
	format!("fn::scale({value}, {factor})")
}

fn main() {
	let _ = add(1, 2);
	let _ = greet("world");
	let _ = get_user("user:abc".to_string());
	let _ = toggle(true);
	let _ = scale(1.0, 2.5);
}
