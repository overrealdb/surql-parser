-- ═══════════════════════════════════════════════════════════
-- Functions — overrealdb platform
-- ═══════════════════════════════════════════════════════════

-- Call user-defined functions
RETURN fn::agent::by_role('deployer');
RETURN fn::project::summary(project:surql_parser);
RETURN fn::deployment::history(project:surql_parser);

-- Built-in string functions
SELECT string::uppercase(name) AS upper_name FROM agent;
SELECT string::len(content) AS spec_size FROM spec;
SELECT string::slug(name) AS slug FROM project;

-- Built-in time functions
RETURN time::now();
SELECT time::group(created_at, 'day') AS day, count() AS deploys
	FROM deployment
	GROUP BY day;

-- Built-in math functions
SELECT math::mean(duration_ms) AS avg_duration FROM sync_event;
SELECT math::max(version) AS latest FROM migration WHERE project = project:surql_parser;

-- Built-in array functions
SELECT array::len(missing_tables) AS count FROM schema_diff WHERE drift_detected = true;
SELECT array::distinct(->manages->project.name) AS unique_projects FROM agent;

-- Crypto functions (for checksums)
SELECT crypto::sha256(content) AS hash FROM spec;
