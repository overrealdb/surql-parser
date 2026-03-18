//! Sample project demonstrating surql-parser build helpers and surql-macros.

// ─── Generated constants from build.rs ───
// Contains: FN_GET_USER, FN_CREATE_POST, FN_USER_LIST, FN_USER_BY_AGE
include!(concat!(env!("OUT_DIR"), "/surql_functions.rs"));

// ─── Compile-time validated queries via surql_check! ───

use surql_macros::{surql_check, surql_function};

pub const QUERY_ALL_USERS: &str = surql_check!("SELECT * FROM user");
pub const QUERY_BY_AGE: &str = surql_check!("SELECT * FROM user WHERE age > 18");
pub const QUERY_USER_POSTS: &str = surql_check!("SELECT *, author.name AS author_name FROM post");
pub const QUERY_MULTI: &str = surql_check!(
	"SELECT name FROM user WHERE age > 21; SELECT title, created_at FROM post ORDER BY created_at DESC"
);
pub const QUERY_DEFINE: &str = surql_check!("DEFINE TABLE audit SCHEMALESS");

// ─── Compile-time validated function wrappers via #[surql_function] ───

#[surql_function("fn::get_user")]
pub fn get_user_call(id: &str) -> String {
	format!("fn::get_user('{id}')")
}

#[surql_function("fn::create_post")]
pub fn create_post_call(title: &str, content: &str, author_id: &str) -> String {
	format!("fn::create_post('{title}', '{content}', {author_id})")
}

#[surql_function("fn::user::list")]
pub fn list_users_call() -> &'static str {
	"fn::user::list()"
}

#[surql_function("fn::user::by_age")]
pub fn users_by_age_call(min: i64, max: i64) -> String {
	format!("fn::user::by_age({min}, {max})")
}
