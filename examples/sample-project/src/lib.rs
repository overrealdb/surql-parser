//! Sample project demonstrating surql-parser build helpers and surql-macros.

// ─── Generated constants from build.rs ───
// Contains: FN_PROJECT_SUMMARY, FN_MIGRATION_APPLY, FN_SYNC_RECORD, etc.
include!(concat!(env!("OUT_DIR"), "/surql_functions.rs"));

// ─── Compile-time validated queries via surql_check! ───

use surql_macros::{surql_check, surql_function, surql_query};

pub const QUERY_ALL_AGENTS: &str = surql_check!("SELECT * FROM agent WHERE active = true");
pub const QUERY_PROJECTS: &str = surql_check!("SELECT * FROM project");
pub const QUERY_PENDING: &str =
	surql_check!("SELECT * FROM migration WHERE status = 'pending' ORDER BY version ASC");

// ─── Compile-time validated queries with parameter checking via surql_query! ───

pub const QUERY_AGENT_BY_ROLE: &str = surql_query!(
	"SELECT * FROM agent WHERE role = $role AND active = true",
	role
);
pub const QUERY_MIGRATIONS: &str = surql_query!(
	"SELECT * FROM migration WHERE project = $project AND version >= $min_version",
	project,
	min_version
);
pub const QUERY_CREATE_AGENT: &str =
	surql_query!("CREATE agent SET name = $name, role = $role", name, role);

// ─── Relations & graph traversal ───

pub const QUERY_RELATE_MANAGES: &str = surql_query!(
	"RELATE $agent->manages->$project SET role = $role, since = time::now()",
	agent,
	project,
	role
);
pub const QUERY_AGENT_PROJECTS: &str =
	surql_check!("SELECT ->manages->project.name AS projects FROM agent:overseer");
pub const QUERY_PROJECT_MANAGERS: &str =
	surql_check!("SELECT <-manages<-agent.name AS managers FROM project:surql_parser");

// ─── Complex queries: deployments, analytics, diffs ───

pub const QUERY_DEPLOYMENT_HISTORY: &str = surql_query!(
	"SELECT *, agent.name AS deployer FROM deployment WHERE project = $project ORDER BY started_at DESC LIMIT $limit",
	project,
	limit
);

pub const QUERY_SCHEMA_DRIFT: &str = surql_check!(
	"SELECT spec.project.name AS project, drift_detected, missing_tables, extra_tables, verified_at FROM schema_diff WHERE drift_detected = true ORDER BY verified_at DESC"
);

pub const QUERY_SYNC_STATS: &str = surql_query!(
	"SELECT action, count() AS total, math::mean(duration_ms) AS avg_ms FROM sync_event WHERE project = $project AND created_at > $since GROUP BY action",
	project,
	since
);

// Transaction: atomic deployment
pub const QUERY_DEPLOY: &str = surql_check!(
	"BEGIN TRANSACTION; LET $d = CREATE deployment SET project = project:surql_parser, agent = agent:overseer, spec = spec:v1; UPDATE migration SET status = 'applied', applied_at = time::now() WHERE project = project:surql_parser AND status = 'pending'; UPDATE $d SET status = 'success', finished_at = time::now(); COMMIT TRANSACTION"
);

// Live query: watch for failed syncs
pub const QUERY_LIVE_FAILURES: &str =
	surql_check!("LIVE SELECT * FROM sync_event WHERE success = false");

// Schema definition
pub const QUERY_DEFINE_SPEC: &str = surql_check!(
	"DEFINE TABLE spec SCHEMAFULL; DEFINE FIELD project ON spec TYPE record<project>; DEFINE FIELD version ON spec TYPE int; DEFINE FIELD content ON spec TYPE string; DEFINE FIELD checksum ON spec TYPE string; DEFINE INDEX spec_version ON spec FIELDS project, version UNIQUE"
);

// ─── Compile-time validated function wrappers via #[surql_function] ───

// With schema validation: param count is verified against DEFINE FUNCTION in surql/
#[surql_function("fn::project::summary", schema = "surql/")]
pub fn project_summary(id: &str) -> String {
	format!("fn::project::summary({id})")
}

#[surql_function("fn::migration::apply", schema = "surql/")]
pub fn migration_apply(mig_id: &str, agent_id: &str) -> String {
	format!("fn::migration::apply({mig_id}, {agent_id})")
}

#[surql_function("fn::agent::by_role", schema = "surql/")]
pub fn agent_by_role(role: &str) -> String {
	format!("fn::agent::by_role('{role}')")
}

// Without schema validation: intentionally passes fewer args (partial binding)
#[surql_function("fn::sync::record")]
pub fn sync_record(project_id: &str, action: &str) -> String {
	format!("fn::sync::record({project_id}, '{action}', 0, 0, true)")
}

// ─── mode = "query": auto-generated function bodies from schema ───
// The macro reads DEFINE FUNCTION params and generates "RETURN fn::name($p1, $p2, ...)"

#[surql_function("fn::agent::by_role", schema = "surql/", mode = "query")]
pub fn q_agent_by_role(_role: &str) -> &'static str {
	unreachable!()
}

#[surql_function("fn::migration::apply", schema = "surql/", mode = "query")]
pub fn q_migration_apply(_mig_id: &str, _agent_id: &str) -> &'static str {
	unreachable!()
}

#[surql_function("fn::deployment::history", schema = "surql/", mode = "query")]
pub fn q_deployment_history(_project_id: &str) -> &'static str {
	unreachable!()
}
