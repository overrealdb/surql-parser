-- ═══════════════════════════════════════════════════════════
-- overrealdb platform schema
-- ═══════════════════════════════════════════════════════════

-- Projects managed by the platform
DEFINE TABLE OVERWRITE project TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
DEFINE FIELD OVERWRITE name ON project TYPE string;
DEFINE FIELD OVERWRITE description ON project TYPE NONE | string;
DEFINE FIELD OVERWRITE owner ON project TYPE record<agent>;
DEFINE FIELD OVERWRITE NS ON project TYPE string COMMENT "SurrealDB namespace";
DEFINE FIELD OVERWRITE DB ON project TYPE string COMMENT "SurrealDB database";
DEFINE FIELD OVERWRITE endpoint ON project TYPE NONE | string COMMENT "ws://host:port";
DEFINE FIELD OVERWRITE created_at ON project TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON project TYPE NONE | datetime;
DEFINE INDEX OVERWRITE project_name ON project FIELDS name UNIQUE;

-- AI agents that operate on projects
DEFINE TABLE OVERWRITE agent TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
DEFINE FIELD OVERWRITE name ON agent TYPE string;
DEFINE FIELD OVERWRITE role ON agent TYPE string COMMENT "developer | reviewer | deployer";
DEFINE FIELD OVERWRITE model ON agent TYPE NONE | string COMMENT "claude-opus-4-6 | gpt-5.1";
DEFINE FIELD OVERWRITE active ON agent TYPE bool DEFAULT true;
DEFINE FIELD OVERWRITE last_seen ON agent TYPE NONE | datetime;
DEFINE INDEX OVERWRITE agent_name ON agent FIELDS name UNIQUE;

-- Schema specifications (overspec)
DEFINE TABLE OVERWRITE spec TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
DEFINE FIELD OVERWRITE project ON spec TYPE record<project>;
DEFINE FIELD OVERWRITE version ON spec TYPE int;
DEFINE FIELD OVERWRITE CONTENT ON spec TYPE string COMMENT "SurrealQL schema definition";
DEFINE FIELD OVERWRITE checksum ON spec TYPE string;
DEFINE FIELD OVERWRITE status ON spec TYPE string DEFAULT 'draft' COMMENT "draft | approved | applied";
DEFINE FIELD OVERWRITE created_by ON spec TYPE record<agent>;
DEFINE FIELD OVERWRITE created_at ON spec TYPE datetime DEFAULT time::now();
DEFINE INDEX OVERWRITE spec_version ON spec FIELDS project, version UNIQUE;

-- Migrations (overshift)
DEFINE TABLE OVERWRITE migration TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
DEFINE FIELD OVERWRITE project ON migration TYPE record<project>;
DEFINE FIELD OVERWRITE version ON migration TYPE int;
DEFINE FIELD OVERWRITE name ON migration TYPE string;
DEFINE FIELD OVERWRITE CONTENT ON migration TYPE string;
DEFINE FIELD OVERWRITE checksum ON migration TYPE string;
DEFINE FIELD OVERWRITE applied_at ON migration TYPE NONE | datetime;
DEFINE FIELD OVERWRITE applied_by ON migration TYPE NONE | record<agent>;
DEFINE FIELD OVERWRITE status ON migration TYPE string DEFAULT 'pending' COMMENT "pending | applied | failed | rolled_back";
DEFINE FIELD OVERWRITE error_message ON migration TYPE NONE | string;
DEFINE INDEX OVERWRITE migration_version ON migration FIELDS project, version UNIQUE;

-- Sync events (oversync)
DEFINE TABLE OVERWRITE sync_event TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
DEFINE FIELD OVERWRITE project ON sync_event TYPE record<project>;
DEFINE FIELD OVERWRITE source ON sync_event TYPE string COMMENT "upstream | local | shadow";
DEFINE FIELD OVERWRITE action ON sync_event TYPE string COMMENT "pull | push | verify | diff";
DEFINE FIELD OVERWRITE changes ON sync_event TYPE int DEFAULT 0;
DEFINE FIELD OVERWRITE duration_ms ON sync_event TYPE int;
DEFINE FIELD OVERWRITE success ON sync_event TYPE bool;
DEFINE FIELD OVERWRITE error ON sync_event TYPE NONE | string;
DEFINE FIELD OVERWRITE created_at ON sync_event TYPE datetime DEFAULT time::now();

-- Schema schema_diff results
DEFINE TABLE OVERWRITE DIFF TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
DEFINE FIELD OVERWRITE spec ON DIFF TYPE record<spec>;
DEFINE FIELD OVERWRITE missing_tables ON DIFF TYPE array<string> DEFAULT [];
DEFINE FIELD OVERWRITE extra_tables ON DIFF TYPE array<string> DEFAULT [];
DEFINE FIELD OVERWRITE missing_functions ON DIFF TYPE array<string> DEFAULT [];
DEFINE FIELD OVERWRITE extra_functions ON DIFF TYPE array<string> DEFAULT [];
DEFINE FIELD OVERWRITE drift_detected ON DIFF TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE verified_at ON DIFF TYPE datetime DEFAULT time::now();

-- Deployment logs
DEFINE TABLE OVERWRITE deployment TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
DEFINE FIELD OVERWRITE project ON deployment TYPE record<project>;
DEFINE FIELD OVERWRITE agent ON deployment TYPE record<agent>;
DEFINE FIELD OVERWRITE spec ON deployment TYPE record<spec>;
DEFINE FIELD OVERWRITE migrations_applied ON deployment TYPE int DEFAULT 0;
DEFINE FIELD OVERWRITE modules_applied ON deployment TYPE int DEFAULT 0;
DEFINE FIELD OVERWRITE schema_drift ON deployment TYPE NONE | string;
DEFINE FIELD OVERWRITE status ON deployment TYPE string DEFAULT 'running' COMMENT "running | success | failed";
DEFINE FIELD OVERWRITE started_at ON deployment TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE finished_at ON deployment TYPE NONE | datetime;
