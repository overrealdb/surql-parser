use crate::{collect_surql_files, extract_definitions, read_surql_file};

/// A parameter defined in a SurrealQL DEFINE FUNCTION statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionParam {
	pub name: String,
	pub kind: String,
}

/// Find a DEFINE FUNCTION by name and return its parameters.
///
/// The `fn_name` should include the `fn::` prefix (e.g., `"fn::project::summary"`).
/// Scans all `.surql` files under `schema_dir` for a matching DEFINE FUNCTION.
///
/// Returns `None` if no matching function is found.
/// Returns `Some(params)` with the list of parameter names and types.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// let params = surql_parser::find_function_params(
///     "fn::greet",
///     Path::new("surql/"),
/// ).unwrap();
/// if let Some(params) = params {
///     assert_eq!(params[0].name, "name");
///     assert_eq!(params[0].kind, "string");
/// }
/// ```
pub fn find_function_params(
	fn_name: &str,
	schema_dir: &std::path::Path,
) -> anyhow::Result<Option<Vec<FunctionParam>>> {
	let stripped = fn_name
		.strip_prefix("fn::")
		.ok_or_else(|| anyhow::anyhow!("function name must start with fn::"))?;

	let mut files = Vec::new();
	collect_surql_files(schema_dir, &mut files);
	files.sort();

	for path in &files {
		let content = match read_surql_file(path) {
			Ok(c) => c,
			Err(e) => {
				tracing::warn!("Skipping {}: {e}", path.display());
				continue;
			}
		};
		let defs = match extract_definitions(&content) {
			Ok(d) => d,
			Err(e) => {
				tracing::warn!("Skipping {}: {e}", path.display());
				continue;
			}
		};
		for func in &defs.functions {
			if func.name == stripped {
				use surrealdb_types::{SqlFormat, ToSql};
				let params = func
					.args
					.iter()
					.map(|(name, kind)| {
						let mut kind_str = String::new();
						kind.fmt_sql(&mut kind_str, SqlFormat::SingleLine);
						let clean_name = name.strip_prefix('$').unwrap_or(name).to_string();
						FunctionParam {
							name: clean_name,
							kind: kind_str,
						}
					})
					.collect();
				return Ok(Some(params));
			}
		}
	}

	Ok(None)
}

/// A field defined in a SurrealQL schema with its type information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldType {
	pub table: String,
	pub field: String,
	pub kind: String,
}

/// Find the type of a specific field on a table, scanning `.surql` files under `schema_dir`.
///
/// Returns `None` if no matching DEFINE FIELD is found.
/// Returns `Some(kind_string)` with the SurrealQL type annotation (e.g., "string", "int").
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// let kind = surql_parser::find_field_type(
///     "user", "age",
///     Path::new("surql/"),
/// ).unwrap();
/// if let Some(kind) = kind {
///     assert_eq!(kind, "int");
/// }
/// ```
pub fn find_field_type(
	table: &str,
	field: &str,
	schema_dir: &std::path::Path,
) -> anyhow::Result<Option<String>> {
	let mut files = Vec::new();
	collect_surql_files(schema_dir, &mut files);
	files.sort();

	for path in &files {
		let content = match read_surql_file(path) {
			Ok(c) => c,
			Err(e) => {
				tracing::warn!("Skipping {}: {e}", path.display());
				continue;
			}
		};
		let defs = match extract_definitions(&content) {
			Ok(d) => d,
			Err(e) => {
				tracing::warn!("Skipping {}: {e}", path.display());
				continue;
			}
		};
		for f in &defs.fields {
			use surrealdb_types::{SqlFormat, ToSql};
			let mut tbl_name = String::new();
			f.what.fmt_sql(&mut tbl_name, SqlFormat::SingleLine);
			let mut fld_name = String::new();
			f.name.fmt_sql(&mut fld_name, SqlFormat::SingleLine);

			if tbl_name == table && fld_name == field {
				if let Some(ref kind) = f.field_kind {
					let mut kind_str = String::new();
					kind.fmt_sql(&mut kind_str, SqlFormat::SingleLine);
					return Ok(Some(kind_str));
				}
				return Ok(None);
			}
		}
	}

	Ok(None)
}

/// Collect all field types for all tables from `.surql` files under `schema_dir`.
///
/// Returns a list of `FieldType` structs, each with table name, field name, and type string.
/// Useful for batch lookups when validating multiple parameters at once.
pub fn collect_field_types(schema_dir: &std::path::Path) -> anyhow::Result<Vec<FieldType>> {
	let mut files = Vec::new();
	collect_surql_files(schema_dir, &mut files);
	files.sort();

	let mut result = Vec::new();

	for path in &files {
		let content = match read_surql_file(path) {
			Ok(c) => c,
			Err(e) => {
				tracing::warn!("Skipping {}: {e}", path.display());
				continue;
			}
		};
		let defs = match extract_definitions(&content) {
			Ok(d) => d,
			Err(e) => {
				tracing::warn!("Skipping {}: {e}", path.display());
				continue;
			}
		};
		for f in &defs.fields {
			use surrealdb_types::{SqlFormat, ToSql};
			let mut tbl_name = String::new();
			f.what.fmt_sql(&mut tbl_name, SqlFormat::SingleLine);
			let mut fld_name = String::new();
			f.name.fmt_sql(&mut fld_name, SqlFormat::SingleLine);

			if let Some(ref kind) = f.field_kind {
				let mut kind_str = String::new();
				kind.fmt_sql(&mut kind_str, SqlFormat::SingleLine);
				result.push(FieldType {
					table: tbl_name,
					field: fld_name,
					kind: kind_str,
				});
			}
		}
	}

	Ok(result)
}
