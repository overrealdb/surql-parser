//! Changelog recording — audit trail of all applied migrations and schema modules.

use surql_macros::surql_check;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use tracing::debug;

use crate::Error;

/// Query: create a changelog entry.
const CREATE_CHANGELOG_SQL: &str = surql_check!(
	r#"
	CREATE changelog SET
		version = $version,
		name = $name,
		type = $type,
		checksum = $checksum,
		applied_at = time::now(),
		applied_by = $instance_id
"#
);

/// Record a changelog entry in the `_system.changelog` table.
///
/// The DB connection must already be pointing at the `_system` database.
pub async fn record_entry(
	db: &Surreal<Any>,
	entry_type: &str,
	version: u32,
	name: &str,
	checksum: &str,
	instance_id: &str,
) -> crate::Result<()> {
	db.query(CREATE_CHANGELOG_SQL)
		.bind(("version", version as i64))
		.bind(("name", name.to_string()))
		.bind(("type", entry_type.to_string()))
		.bind(("checksum", checksum.to_string()))
		.bind(("instance_id", instance_id.to_string()))
		.await
		.map_err(|e| Error::Migration(format!("changelog write failed: {e}")))?;

	debug!(entry_type, version, name, "recorded changelog entry");
	Ok(())
}
