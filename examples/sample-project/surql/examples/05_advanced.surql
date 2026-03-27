-- ═══════════════════════════════════════════════════════════
-- Advanced queries — overrealdb platform analytics
-- ═══════════════════════════════════════════════════════════

-- Subquery: projects with most migrations
SELECT
	project.name AS project_name,
	count() AS migration_count
	FROM migration
	WHERE status = 'applied'
	GROUP BY project
	ORDER BY migration_count DESC;

-- Transaction: atomic deployment
BEGIN TRANSACTION;
	LET $deploy = CREATE deployment SET
		project = project:surql_parser,
		agent = agent:overseer,
		spec = spec:v1,
		status = 'running';
	UPDATE migration SET
		status = 'applied',
		applied_at = time::now(),
		applied_by = agent:overseer
		WHERE project = project:surql_parser
		AND status = 'pending';
	UPDATE $deploy SET
		status = 'success',
		finished_at = time::now();
COMMIT TRANSACTION;

-- Live query: watch for new sync events
LIVE SELECT * FROM sync_event
	WHERE success = false;

-- Timeseries: sync events per hour
SELECT
	time::GROUP(created_at, 'hour') AS hour,
	count() AS events,
	math::mean(duration_ms) AS avg_duration
	FROM sync_event  -- surql-allow: undefined-table
	WHERE created_at > time::now() - 24h
	GROUP BY hour
	ORDER BY hour DESC;

-- Deployment success rate
LET $success = SELECT count() AS c FROM deployment WHERE status = 'success' GROUP ALL;
LET $total = SELECT count() AS c FROM deployment GROUP ALL;
RETURN math::round($success[0].c / $total[0].c * 100);

-- Schema drift detection across projects
SELECT
	spec.project.name AS project,
	drift_detected,
	missing_tables,
	extra_tables,
	verified_at
	FROM DIFF
	WHERE drift_detected = true
	ORDER BY verified_at DESC;
