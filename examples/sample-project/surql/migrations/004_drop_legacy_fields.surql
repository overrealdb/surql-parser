-- Migration 004: Add oversync tracking and analytics
-- Track sync events, add search indexes for specs.

DEFINE INDEX spec_search ON spec FIELDS content;

-- Analytics: deployment stats per project
DEFINE FUNCTION fn::stats::project($project_id: record<project>) -> object {
	LET $total_migrations = SELECT count() AS c FROM migration WHERE project = $project_id GROUP ALL;
	LET $applied = SELECT count() AS c FROM migration WHERE project = $project_id AND status = 'applied' GROUP ALL;
	LET $syncs = SELECT count() AS c FROM sync_event WHERE project = $project_id AND success = true GROUP ALL;
	LET $deployments = SELECT count() AS c FROM deployment WHERE project = $project_id GROUP ALL;
	RETURN {
		migrations_total: $total_migrations,
		migrations_applied: $applied,
		syncs_successful: $syncs,
		deployments: $deployments
	};
};
