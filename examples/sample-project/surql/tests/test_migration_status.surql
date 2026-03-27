-- Test: create a migration record and verify status tracking.
CREATE project:demo SET
	name = 'demo-project',
	description = 'Test project',
	ns = 'test',
	db = 'test';

CREATE agent:deployer SET name = 'Deployer', role = 'deployer', active = true;

CREATE migration:first SET
	project = project:demo,
	version = 1,
	description = 'initial schema',
	status = 'applied',
	applied_by = agent:deployer;

LET $mig = SELECT * FROM migration:first;
IF $mig[0].version != 1 { THROW 'Expected migration version 1'; };
IF $mig[0].status != 'applied' { THROW 'Expected migration status applied'; };

-- Verify function calls work
LET $now = time::now();
IF !$now { THROW 'time::now() returned falsy value'; };
