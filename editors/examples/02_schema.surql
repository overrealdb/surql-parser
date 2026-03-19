-- ═══════════════════════════════════════════════════════════
-- Schema definitions — DEFINE statements
-- ═══════════════════════════════════════════════════════════

-- Namespace & Database
DEFINE NAMESPACE production;
DEFINE DATABASE main;
USE NS production DB main;

-- Tables
DEFINE TABLE user SCHEMAFULL;
DEFINE TABLE post SCHEMALESS;
DEFINE TABLE audit SCHEMAFULL DROP;
DEFINE TABLE article SCHEMAFULL CHANGEFEED 1d INCLUDE ORIGINAL;

-- Fields with types
DEFINE FIELD name ON user TYPE string;
DEFINE FIELD email ON user TYPE string ASSERT string::is_email($value);
DEFINE FIELD age ON user TYPE int DEFAULT 0;
DEFINE FIELD active ON user TYPE bool DEFAULT true;
DEFINE FIELD created_at ON user TYPE datetime DEFAULT time::now();
DEFINE FIELD tags ON user TYPE array<string> DEFAULT [];
DEFINE FIELD settings ON user TYPE object DEFAULT {};
DEFINE FIELD avatar ON user TYPE option<string>;
DEFINE FIELD bio ON user TYPE option<object> FLEXIBLE;
DEFINE FIELD password ON user TYPE string READONLY;

-- Record links
DEFINE FIELD author ON post TYPE record<user>;
DEFINE FIELD comments ON post TYPE array<record<comment>>;
DEFINE FIELD category ON post TYPE record<category> DEFAULT category:general;

-- Indexes
DEFINE INDEX user_email ON user FIELDS email UNIQUE;
DEFINE INDEX user_name ON user FIELDS name;
DEFINE INDEX post_created ON post FIELDS created_at;

-- Events
DEFINE EVENT user_created ON user WHEN $event = 'CREATE' THEN {
	CREATE audit SET
		action = 'user_created',
		target = $after.id,
		timestamp = time::now();
};

DEFINE EVENT post_updated ON post WHEN $event = 'UPDATE' THEN {
	UPDATE $after.id SET updated_at = time::now();
};

-- Analyzers
DEFINE ANALYZER ascii TOKENIZERS blank, class FILTERS ascii, lowercase;

-- Access & Users
DEFINE USER admin ON DATABASE PASSWORD 'secret123' ROLES OWNER;

-- Or Replace (overwrite)
DEFINE TABLE IF NOT EXISTS user SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS name ON user TYPE string;

-- If Not Exists
DEFINE TABLE IF NOT EXISTS archive SCHEMALESS;
