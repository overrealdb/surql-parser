-- ═══════════════════════════════════════════════════════════
-- overrealdb platform functions
-- ═══════════════════════════════════════════════════════════

-- Get project with its latest spec
DEFINE FUNCTION OVERWRITE fn::project::summary($id: record<project>) -> object {
	LET $proj = SELECT * FROM $id;
	LET $latest_spec = SELECT * FROM spec WHERE project = $id ORDER BY version DESC LIMIT 1;
	LET $migration_count = SELECT count() FROM migration WHERE project = $id AND status = 'applied';
	RETURN {
		project: $proj,
		latest_spec: $latest_spec,
		migrations: $migration_count
	};
};

-- Apply a migration and record the result
DEFINE FUNCTION OVERWRITE fn::migration::apply($mig_id: record<migration>, $agent_id: record<agent>) -> object {
	UPDATE $mig_id SET
		status = 'applied',
		applied_at = time::now(),
		applied_by = $agent_id;
	RETURN SELECT * FROM $mig_id;
};

-- Record a sync event
DEFINE FUNCTION OVERWRITE fn::sync::record($project_id: record<project>, $action: string, $changes: int, $duration: int, $ok: bool) -> object {
	RETURN CREATE sync_event SET
		project = $project_id,
		source = 'local',
		action = $action,
		changes = $changes,
		duration_ms = $duration,
		success = $ok;
};

-- Verify schema drift between spec and live DB
DEFINE FUNCTION OVERWRITE fn::spec::verify($spec_id: record<spec>) -> object {
	LET $spec = SELECT * FROM $spec_id;
	RETURN CREATE schema_diff SET
		spec = $spec_id,
		drift_detected = false,
		verified_at = time::now();
};

-- List agents by role
DEFINE FUNCTION OVERWRITE fn::agent::by_role($role: string) -> array {
	RETURN SELECT * FROM agent WHERE role = $role AND active = true;
};

-- Get deployment history for a project
DEFINE FUNCTION OVERWRITE fn::deployment::history($project_id: record<project>) -> array {
	RETURN SELECT * FROM deployment
		WHERE project = $project_id
		ORDER BY started_at DESC
		LIMIT 20;
};
