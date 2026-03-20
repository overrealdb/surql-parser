//! Schema validation — verify that all expected functions exist in the database.

use surql_macros::surql_check;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use tracing::{debug, info};

use crate::Error;

/// Query: retrieve database info (tables, functions, etc.).
const INFO_FOR_DB_SQL: &str = surql_check!("INFO FOR DB");

/// Validate that all expected functions exist in the database after schema apply.
///
/// Queries `INFO FOR DB` and checks that each expected function name is present.
pub async fn validate_functions(db: &Surreal<Any>, expected: &[String]) -> crate::Result<()> {
	if expected.is_empty() {
		return Ok(());
	}

	info!(
		count = expected.len(),
		"validating functions exist in database"
	);

	let mut response = db
		.query(INFO_FOR_DB_SQL)
		.await
		.map_err(|e| Error::Validation(format!("INFO FOR DB failed: {e}")))?;

	let info: Option<serde_json::Value> = response
		.take(0)
		.map_err(|e| Error::Validation(format!("failed to read DB info: {e}")))?;

	let defined_functions: Vec<String> = if let Some(info) = &info {
		info.get("functions")
			.and_then(|f| f.as_object())
			.map(|obj| obj.keys().cloned().collect())
			.unwrap_or_default()
	} else {
		Vec::new()
	};

	debug!(?defined_functions, "functions found in database");

	let mut missing = Vec::new();
	for func in expected {
		let fn_key = format!("fn::{func}");
		let found = defined_functions.iter().any(|f| f == &fn_key || f == func);
		if !found {
			missing.push(func.clone());
		}
	}

	if !missing.is_empty() {
		return Err(Error::Validation(format!(
			"missing functions in database: {}",
			missing
				.iter()
				.map(|f| format!("fn::{f}"))
				.collect::<Vec<_>>()
				.join(", "),
		)));
	}

	info!("all {} functions validated", expected.len());
	Ok(())
}
