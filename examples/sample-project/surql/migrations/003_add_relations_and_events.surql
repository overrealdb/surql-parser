-- Migration 003: Add relations and events
-- Agents manage projects, deployments trigger sync events.

-- Agent manages project (graph edge)
DEFINE TABLE manages TYPE RELATION SCHEMAFULL PERMISSIONS NONE;
DEFINE FIELD IN ON manages TYPE record<agent>;
DEFINE FIELD out ON manages TYPE record<project>;
DEFINE FIELD role ON manages TYPE string COMMENT "owner | maintainer | viewer";
DEFINE FIELD since ON manages TYPE datetime DEFAULT time::now();

RELATE agent:overseer->manages->project:surql_parser SET role = 'owner';
RELATE agent:reviewer->manages->project:surql_parser SET role = 'maintainer';

-- Audit log (DROP = auto-delete old records)
DEFINE TABLE audit TYPE NORMAL DROP SCHEMAFULL PERMISSIONS NONE;
DEFINE FIELD action ON audit TYPE string;
DEFINE FIELD target ON audit TYPE record;
DEFINE FIELD agent ON audit TYPE NONE | record<agent>;
DEFINE FIELD details ON audit TYPE NONE | object;
DEFINE FIELD created_at ON audit TYPE datetime DEFAULT time::now();

-- Events: log deployments and migrations
DEFINE EVENT deployment_started ON deployment
	WHEN $event = 'CREATE'
	THEN {
		CREATE audit SET
			action = 'deployment_started',
			target = $after.id,
			agent = $after.agent,
			created_at = time::now();
	};

DEFINE EVENT migration_applied ON migration
	WHEN $event = 'UPDATE' AND $after.status = 'applied' AND $before.status = 'pending'
	THEN {
		CREATE audit SET
			action = 'migration_applied',
			target = $after.id,
			agent = $after.applied_by,
			details = { version: $after.version, name: $after.name },
			created_at = time::now();
	};
