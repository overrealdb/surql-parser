-- Migration 002: Add overshift migration tracking
-- Seeds initial specs and migration records for the parser project.

CREATE spec:v1 SET
	project = project:surql_parser,
	version = 1,
	content = 'DEFINE TABLE project SCHEMAFULL; DEFINE TABLE agent SCHEMAFULL;',
	checksum = 'a1b2c3d4e5f6',
	status = 'applied',
	created_by = agent:overseer;

CREATE migration:v001 SET
	project = project:surql_parser,
	version = 1,
	name = 'bootstrap',
	content = 'CREATE agent:overseer ...',
	checksum = 'f6e5d4c3b2a1',
	status = 'applied',
	applied_at = time::now(),
	applied_by = agent:overseer;
