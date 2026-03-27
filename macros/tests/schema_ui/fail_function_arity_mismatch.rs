use surql_macros::surql_function;

#[surql_function("fn::add", schema = "surql_fixtures/")]
fn add(a: i64) -> String {
	format!("fn::add({a})")
}

fn main() {}
