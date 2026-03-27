-- ═══════════════════════════════════════════════════════════
-- Basic queries — overrealdb platform
-- ═══════════════════════════════════════════════════════════

-- List all active agents
SELECT * FROM agent WHERE active = true;

-- Get a specific project
SELECT * FROM project:surql_parser;

-- Find all migrations for a project
SELECT * FROM migration
	WHERE project = project:surql_parser
	ORDER BY version ASC;

-- Count pending migrations
SELECT count() AS pending FROM migration
	WHERE status = 'pending'
	GROUP ALL;

-- Get latest spec
SELECT * FROM spec
	WHERE project = project:surql_parser
	ORDER BY version DESC
	LIMIT 1;

-- Simple parameter query
LET $min_version = 2;
SELECT * FROM migration
	WHERE version >= $min_version
	AND status = 'applied';
