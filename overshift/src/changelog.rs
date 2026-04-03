//! Changelog recording — audit trail of all applied migrations and schema modules.

use std::collections::HashMap;

use surql_macros::surql_check;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use tracing::{debug, warn};

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

const READ_SCHEMA_CHECKSUMS_SQL: &str = surql_check!(
	r#"
	SELECT name, checksum, applied_at
	FROM changelog
	WHERE type = 'schema_module'
	ORDER BY applied_at DESC
"#
);

/// Read the latest recorded checksum for each schema module from `_system.changelog`.
///
/// The DB connection must already be pointing at the `_system` database.
pub async fn read_schema_checksums(db: &Surreal<Any>) -> crate::Result<HashMap<String, String>> {
	let mut response = db
		.query(READ_SCHEMA_CHECKSUMS_SQL)
		.await
		.map_err(|e| Error::Migration(format!("read schema checksums failed: {e}")))?;

	let rows: Vec<serde_json::Value> = response
		.take(0)
		.map_err(|e| Error::Migration(format!("take schema checksums failed: {e}")))?;

	let mut checksums = HashMap::new();
	for row in &rows {
		let Some(name) = row.get("name").and_then(|v| v.as_str()) else {
			warn!(?row, "changelog row missing string 'name' field");
			continue;
		};
		let Some(checksum) = row.get("checksum").and_then(|v| v.as_str()) else {
			warn!(name, "changelog row missing string 'checksum' field");
			continue;
		};
		// Ordered by applied_at DESC — first entry per name is the latest
		checksums
			.entry(name.to_string())
			.or_insert_with(|| checksum.to_string());
	}

	debug!(
		count = checksums.len(),
		"read schema module checksums from changelog"
	);
	Ok(checksums)
}
