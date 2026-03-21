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

/// Diff between expected and actual database state.
#[derive(Debug, Default)]
pub struct SchemaDiff {
	pub missing_tables: Vec<String>,
	pub extra_tables: Vec<String>,
	pub missing_functions: Vec<String>,
	pub extra_functions: Vec<String>,
}

impl SchemaDiff {
	pub fn is_empty(&self) -> bool {
		self.missing_tables.is_empty()
			&& self.extra_tables.is_empty()
			&& self.missing_functions.is_empty()
			&& self.extra_functions.is_empty()
	}
}

impl std::fmt::Display for SchemaDiff {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.is_empty() {
			return write!(f, "Schema matches");
		}
		for t in &self.missing_tables {
			writeln!(f, "- table `{t}` missing in target")?;
		}
		for t in &self.extra_tables {
			writeln!(f, "+ table `{t}` extra in target")?;
		}
		for func in &self.missing_functions {
			writeln!(f, "- function `{func}` missing in target")?;
		}
		for func in &self.extra_functions {
			writeln!(f, "+ function `{func}` extra in target")?;
		}
		Ok(())
	}
}

/// Extract table and function names from INFO FOR DB response.
fn extract_db_info(info: &serde_json::Value) -> (Vec<String>, Vec<String>) {
	let tables = info
		.get("tables")
		.and_then(|t| t.as_object())
		.map(|obj| obj.keys().cloned().collect())
		.unwrap_or_default();
	let functions = info
		.get("functions")
		.and_then(|f| f.as_object())
		.map(|obj| obj.keys().cloned().collect())
		.unwrap_or_default();
	(tables, functions)
}

/// Compare two database states and return the diff.
pub fn compare_db_info(expected: &serde_json::Value, actual: &serde_json::Value) -> SchemaDiff {
	let (exp_tables, exp_fns) = extract_db_info(expected);
	let (act_tables, act_fns) = extract_db_info(actual);

	SchemaDiff {
		missing_tables: exp_tables
			.iter()
			.filter(|t| !act_tables.contains(t))
			.cloned()
			.collect(),
		extra_tables: act_tables
			.iter()
			.filter(|t| !exp_tables.contains(t))
			.cloned()
			.collect(),
		missing_functions: exp_fns
			.iter()
			.filter(|f| !act_fns.contains(f))
			.cloned()
			.collect(),
		extra_functions: act_fns
			.iter()
			.filter(|f| !exp_fns.contains(f))
			.cloned()
			.collect(),
	}
}

/// Query INFO FOR DB from a database connection.
pub async fn query_db_info(db: &Surreal<Any>) -> crate::Result<serde_json::Value> {
	let mut response = db
		.query(INFO_FOR_DB_SQL)
		.await
		.map_err(|e| Error::Validation(format!("INFO FOR DB failed: {e}")))?;
	let info: Option<serde_json::Value> = response
		.take(0)
		.map_err(|e| Error::Validation(format!("failed to read DB info: {e}")))?;
	Ok(info.unwrap_or_default())
}
