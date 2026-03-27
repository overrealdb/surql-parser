use surql_macros::surql_function;

#[surql_function("fn::greet", schema = "surql_fixtures/")]
fn greet(name: i64) -> String {
	format!("fn::greet({name})")
}

fn main() {}
