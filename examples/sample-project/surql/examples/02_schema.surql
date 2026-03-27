-- ═══════════════════════════════════════════════════════════
-- Schema definitions — overrealdb platform DEFINE statements
-- ═══════════════════════════════════════════════════════════

-- Table with full schema enforcement
DEFINE TABLE project TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

-- Fields with types, defaults, and constraints
DEFINE FIELD name ON project TYPE string
	ASSERT string::len($value) >= 2 AND string::len($value) <= 64;
DEFINE FIELD endpoint ON project TYPE none | string;
DEFINE FIELD created_at ON project TYPE datetime DEFAULT time::now();
DEFINE FIELD settings ON project TYPE object DEFAULT {};
DEFINE FIELD settings.auto_deploy ON project TYPE bool DEFAULT false;
DEFINE FIELD settings.verify_after_apply ON project TYPE bool DEFAULT true;

-- Unique and search indexes
DEFINE INDEX project_name ON project FIELDS name UNIQUE;
DEFINE INDEX project_desc ON project FIELDS description;

-- Agent with READONLY password
DEFINE TABLE agent TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
DEFINE FIELD name ON agent TYPE string;
DEFINE FIELD api_key ON agent TYPE string READONLY;
DEFINE FIELD role ON agent TYPE string
	ASSERT $value IN ['developer', 'reviewer', 'deployer', 'admin'];

-- Changefeed for audit
DEFINE TABLE migration TYPE NORMAL SCHEMAFULL
	CHANGEFEED 7d INCLUDE ORIGINAL
	PERMISSIONS NONE;
