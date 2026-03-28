use surql_macros::surql_function;

#[surql_function("fn::greet", schema = "surql_fixtures/", mode = "query")]
fn q_greet(_name: i64) -> &'static str {
	unreachable!()
}

fn main() {}
