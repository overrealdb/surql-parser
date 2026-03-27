-- ═══════════════════════════════════════════════════════════
-- Relations & Graph traversal — overrealdb platform
-- ═══════════════════════════════════════════════════════════

-- Create management relations
RELATE agent:overseer->manages->project:surql_parser SET role = 'owner';
RELATE agent:reviewer->manages->project:surql_parser SET role = 'maintainer';

-- Graph traversal: which projects does an agent manage?
SELECT ->manages->project.name AS projects FROM agent:overseer;

-- Reverse: who manages this project?
SELECT <-manages<-agent.name AS managers FROM project:surql_parser;

-- Graph with edge properties
SELECT ->manages.role AS roles, ->manages->project.name AS projects
	FROM agent:overseer;

-- Find all deployer agents and their projects
SELECT name, ->manages->project.* AS projects
	FROM agent
	WHERE role = 'deployer';

-- Multi-hop: agent → manages → project → has migrations
SELECT
	->manages->project.name AS project,
	->manages->project->migration.name AS migrations
	FROM agent:overseer;

-- Relate with CONTENT
RELATE agent:reviewer->manages->project:surql_parser CONTENT {
	role: 'reviewer',
	since: time::now(),
};
