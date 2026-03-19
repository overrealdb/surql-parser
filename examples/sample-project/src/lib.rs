//! Sample project demonstrating surql-parser build helpers and surql-macros.

// ─── Generated constants from build.rs ───
// Contains: FN_GET_USER, FN_CREATE_POST, FN_USER_LIST, FN_USER_BY_AGE
include!(concat!(env!("OUT_DIR"), "/surql_functions.rs"));

// ─── Compile-time validated queries via surql_check! ───

use surql_macros::{surql_check, surql_function, surql_query};

pub const QUERY_ALL_USERS: &str = surql_check!("SELECT * FROM user");
pub const QUERY_BY_AGE: &str = surql_check!("SELECT * FROM user WHERE age > 18");

// ─── Compile-time validated queries with parameter checking via surql_query! ───

pub const QUERY_PARAMETERIZED: &str = surql_query!(
	"SELECT * FROM user WHERE age > $min AND name = $name",
	min,
	name
);
pub const QUERY_NO_PARAMS: &str = surql_query!("SELECT * FROM user");
pub const QUERY_SINGLE_PARAM: &str = surql_query!("CREATE user SET name = $name", name);
pub const QUERY_USER_POSTS: &str = surql_check!("SELECT *, author.name AS author_name FROM post");
pub const QUERY_MULTI: &str = surql_check!(
	"SELECT name FROM user WHERE age > 21; SELECT title, created_at FROM post ORDER BY created_at DESC"
);
pub const QUERY_DEFINE: &str = surql_check!("DEFINE TABLE audit SCHEMALESS");

// ─── Relations & graph traversal (SurrealDB 3+ rich syntax) ───

pub const QUERY_RELATE: &str = surql_query!(
	"RELATE $from->follows->$to SET since = time::now()",
	from,
	to
);
pub const QUERY_GRAPH: &str = surql_query!(
	"SELECT ->follows->user.name AS friends FROM type::record('user', $id)",
	id
);
pub const QUERY_REVERSE_GRAPH: &str =
	surql_check!("SELECT <-follows<-user.name AS followers FROM user:tobie");
pub const QUERY_MULTI_HOP: &str =
	surql_check!("SELECT ->follows->user->follows->user.name AS fof FROM user:tobie");
pub const QUERY_RELATION_TABLE: &str =
	surql_check!("DEFINE TABLE follows TYPE RELATION FROM user TO user ENFORCED SCHEMAFULL");
pub const QUERY_RELATION_FIELD: &str =
	surql_check!("DEFINE FIELD since ON follows TYPE datetime DEFAULT time::now()");
pub const QUERY_RECORD_LINK: &str = surql_query!(
	"CREATE post SET title = $title, author = $author",
	title,
	author
);
pub const QUERY_SUBQUERY_RELATE: &str = surql_check!(
	"LET $users = SELECT id FROM user WHERE age > 21; RELATE $users->attends->event:conference"
);

// ─── Complex queries: graphs, subqueries, aggregations, timeseries ───

// Deep graph traversal with filtering at each hop
pub const QUERY_DEEP_GRAPH_FILTER: &str = surql_check!(
	"SELECT ->knows->(? WHERE age > 21)->knows->(? WHERE country = 'US').name AS connections FROM user:tobie"
);

// Aggregation with GROUP BY on graph results
pub const QUERY_GRAPH_AGGREGATION: &str = surql_check!(
	"SELECT count() AS total, country, array::distinct(->knows->user.name) AS contacts FROM user GROUP BY country"
);

// Nested subquery with RELATE + graph in single transaction
pub const QUERY_TRANSACTION: &str = surql_check!(
	"BEGIN TRANSACTION; LET $u = CREATE user SET name = 'Alice', age = 30; LET $p = CREATE post SET title = 'Hello'; RELATE $u->wrote->$p SET created_at = time::now(); COMMIT TRANSACTION"
);

// Record links + deep field access
pub const QUERY_DEEP_FIELD_ACCESS: &str = surql_check!(
	"SELECT title, author.name AS author_name, created_at FROM post ORDER BY created_at DESC LIMIT 10"
);

// Complex WHERE with math functions + type coercion
pub const QUERY_MATH_FILTER: &str = surql_query!(
	"SELECT *, math::sqrt(math::pow(lat - $lat, 2) + math::pow(lon - $lon, 2)) AS distance FROM location ORDER BY distance LIMIT $limit",
	lat,
	lon,
	limit
);

// Timeseries: range scan with datetime arithmetic
pub const QUERY_TIMESERIES_RANGE: &str = surql_query!(
	"SELECT time::group(created_at, 'hour') AS hour, count() AS events, math::mean(value) AS avg_value FROM metric WHERE created_at > $since AND created_at < time::now() GROUP BY hour ORDER BY hour",
	since
);

// Recursive graph: find all paths between nodes
pub const QUERY_RECURSIVE_GRAPH: &str =
	surql_check!("SELECT ->knows->user->knows->user->knows->user AS path FROM user:tobie");

// Mixed relation types + conditional edge traversal
pub const QUERY_MIXED_RELATIONS: &str = surql_check!(
	"SELECT id, ->wrote->post.title AS posts, ->follows->user.name AS following, <-follows<-user.name AS followers, ->attends->event.{ title, date } AS events FROM user:tobie"
);

// UPSERT with complex SET + record links
pub const QUERY_UPSERT_COMPLEX: &str = surql_query!(
	"UPSERT user SET name = $name, email = $email, updated_at = time::now(), settings = { theme: 'dark', lang: $lang }, tags = array::union(tags, $new_tags)",
	name,
	email,
	lang,
	new_tags
);

// Subquery as field value + conditional expressions
pub const QUERY_SUBQUERY_FIELDS: &str = surql_check!(
	"SELECT *, (SELECT count() FROM ->wrote->post) AS post_count, IF age >= 18 THEN 'adult' ELSE 'minor' END AS category FROM user"
);

// Schema definition: full table with fields, indexes, events
pub const QUERY_FULL_SCHEMA: &str = surql_check!(
	"DEFINE TABLE article SCHEMAFULL; DEFINE FIELD title ON article TYPE string ASSERT string::len($value) > 0; DEFINE FIELD body ON article TYPE string; DEFINE FIELD author ON article TYPE record<user>; DEFINE FIELD tags ON article TYPE array<string> DEFAULT []; DEFINE FIELD created_at ON article TYPE datetime DEFAULT time::now(); DEFINE INDEX article_author_idx ON article FIELDS author; DEFINE EVENT article_created ON article WHEN $event = 'CREATE' THEN { CREATE audit SET action = 'article_created', target = $after.id }"
);

// Live query with diff
pub const QUERY_LIVE: &str = surql_check!("LIVE SELECT DIFF FROM user WHERE age > 18");

// Complex record ID generation + insert
pub const QUERY_INSERT_BATCH: &str = surql_check!(
	"INSERT INTO temperature [{ device: 'sensor-1', value: 22.5, ts: time::now() }, { device: 'sensor-2', value: 23.1, ts: time::now() }]"
);

// Changefeed
pub const QUERY_DEFINE_CHANGEFEED: &str =
	surql_check!("DEFINE TABLE reading CHANGEFEED 1d INCLUDE ORIGINAL SCHEMALESS");

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
