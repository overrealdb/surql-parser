-- ═══════════════════════════════════════════════════════════
-- Real-world scenario — overrealdb deployment pipeline
-- ═══════════════════════════════════════════════════════════

-- 1. Create a new project
CREATE project:overshift SET
	name = 'overshift',
	description = 'Migration engine for the overrealdb ecosystem',
	owner = agent:overseer,
	ns = 'overrealdb',
	db = 'overshift',
	endpoint = 'ws://localhost:8000';

-- 2. Agent takes ownership
RELATE agent:overseer->manages->project:overshift SET
	role = 'owner',
	since = time::now();

-- 3. Create initial spec
CREATE spec:overshift_v1 SET
	project = project:overshift,
	version = 1,
	content = 'DEFINE TABLE migration SCHEMAFULL;',
	checksum = crypto::sha256('DEFINE TABLE migration SCHEMAFULL;'),
	status = 'approved',
	created_by = agent:overseer;

-- 4. Create and apply migrations
CREATE migration:overshift_001 SET
	project = project:overshift,
	version = 1,
	name = 'bootstrap',
	content = 'DEFINE TABLE migration SCHEMAFULL;',
	checksum = crypto::sha256('DEFINE TABLE migration SCHEMAFULL;'),
	status = 'pending';

-- Apply via function
UPDATE migration:overshift_001 SET
	status = 'applied',
	applied_at = time::now(),
	applied_by = agent:overseer;

-- 5. Record deployment
CREATE deployment:d1 SET
	project = project:overshift,
	agent = agent:overseer,
	spec = spec:overshift_v1,
	migrations_applied = 1,
	modules_applied = 0,
	status = 'success',
	finished_at = time::now();

-- 6. Verify: no drift
CREATE schema_diff:check1 SET
	spec = spec:overshift_v1,
	drift_detected = false;

-- 7. Record sync event
CREATE sync_event SET
	project = project:overshift,
	source = 'local',
	action = 'verify',
	changes = 0,
	duration_ms = 42,
	success = true;

-- 8. Query the full picture
SELECT
	name,
	->manages<-agent.name AS managers,
	(SELECT count() FROM migration WHERE project = project:overshift AND status = 'applied') AS applied_migrations,
	(SELECT count() FROM deployment WHERE project = project:overshift) AS deployments
	FROM project:overshift;
