use surql_macros::surql_function;

#[surql_function("fn::user::get")]
fn user_get(id: i64) -> String {
	format!("fn::user::get({id})")
}

#[surql_function("fn::auth::check_permission")]
fn check_permission(role: &str) -> String {
	format!("fn::auth::check_permission('{role}')")
}

fn main() {
	let _ = user_get(42);
	let _ = check_permission("admin");
}
