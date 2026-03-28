use surql_macros::surql_function;

#[surql_function("fn::greet", schema = "surql_fixtures/", mode = "query")]
fn q_greet(_name: &str) -> &'static str {
	unreachable!()
}

#[surql_function("fn::add", schema = "surql_fixtures/", mode = "query")]
fn q_add(_a: i64, _b: i64) -> &'static str {
	unreachable!()
}

fn main() {
	assert_eq!(q_greet(""), "RETURN fn::greet($name)");
	assert_eq!(q_add(0, 0), "RETURN fn::add($a, $b)");
}
