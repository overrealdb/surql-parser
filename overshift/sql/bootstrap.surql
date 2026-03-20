-- Bootstrap: _system tables for migration tracking
-- Applied idempotently (IF NOT EXISTS) before each migration run

-- Migration tracking
DEFINE TABLE IF NOT EXISTS migration_lock SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS version     ON migration_lock TYPE int;
DEFINE FIELD IF NOT EXISTS applied_at  ON migration_lock TYPE datetime;
DEFINE FIELD IF NOT EXISTS checksum    ON migration_lock TYPE string;
DEFINE FIELD IF NOT EXISTS instance_id ON migration_lock TYPE string;
DEFINE INDEX IF NOT EXISTS idx_ml_version ON migration_lock FIELDS version UNIQUE;

-- Leader election for distributed lock
DEFINE TABLE IF NOT EXISTS leader_lock SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS holder      ON leader_lock TYPE option<string>;
DEFINE FIELD IF NOT EXISTS acquired_at ON leader_lock TYPE datetime;
DEFINE FIELD IF NOT EXISTS expires_at  ON leader_lock TYPE datetime;

-- Generic distributed lock (shedlock pattern)
DEFINE TABLE IF NOT EXISTS shedlock SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS name      ON shedlock TYPE string;
DEFINE FIELD IF NOT EXISTS holder    ON shedlock TYPE option<string>;
DEFINE FIELD IF NOT EXISTS locked_at ON shedlock TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS lock_until ON shedlock TYPE option<datetime>;
DEFINE INDEX IF NOT EXISTS idx_shedlock_name ON shedlock FIELDS name UNIQUE;

-- Changelog (audit trail)
DEFINE TABLE IF NOT EXISTS changelog SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS version    ON changelog TYPE int;
DEFINE FIELD IF NOT EXISTS name       ON changelog TYPE string;
DEFINE FIELD IF NOT EXISTS type       ON changelog TYPE string;
DEFINE FIELD IF NOT EXISTS checksum   ON changelog TYPE string;
DEFINE FIELD IF NOT EXISTS applied_at ON changelog TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS applied_by ON changelog TYPE string;
