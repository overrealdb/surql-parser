//! Build-time SurrealQL schema validation and code generation.
//!
//! Call these functions from your `build.rs` to validate `.surql` files and
//! generate typed Rust constants for SurrealQL functions at compile time.
//!
//! # Example `build.rs`
//!
//! ```rust,no_run
//! let out_dir = std::env::var("OUT_DIR").unwrap();
//! surql_parser::build::validate_schema("surql/");
//! surql_parser::build::generate_typed_functions(
//!     "surql/",
//!     &format!("{out_dir}/surql_functions.rs"),
//! );
//! ```
//!
//! Then in your library:
//!
//! ```rust,ignore
//! include!(concat!(env!("OUT_DIR"), "/surql_functions.rs"));
//! ```

use std::path::Path;

use surrealdb_types::{SqlFormat, ToSql};

/// Validate all `.surql` files in a directory at build time.
///
/// Walks `dir` recursively, parses every `.surql` file, and prints
/// `cargo:warning` for files that fail to parse. Emits
/// `cargo:rerun-if-changed` directives for each file.
///
/// Returns the number of files with errors.
pub fn validate_schema(dir: impl AsRef<Path>) -> usize {
	validate_schema_inner(dir.as_ref(), true)
}

/// How the build should react to SurrealQL validation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
	/// Print `cargo:warning` but don't fail the build.
	Warn,
	/// Panic and fail the build.
	Fail,
	/// Silently ignore errors (only return the count).
	Ignore,
}

/// Validate all `.surql` files with a configurable error mode.
///
/// ```rust,no_run
/// use surql_parser::build::{validate_schema_with, ValidationMode};
/// validate_schema_with("surql/", ValidationMode::Fail);
/// ```
pub fn validate_schema_with(dir: impl AsRef<Path>, mode: ValidationMode) {
	let errors = validate_schema_inner(dir.as_ref(), mode != ValidationMode::Ignore);
	if mode == ValidationMode::Fail && errors > 0 {
		panic!("SurrealQL validation failed: {errors} file(s) with errors");
	}
}

/// Validate all `.surql` files in a directory at build time, failing on errors.
///
/// Calls [`validate_schema`] and panics if any files contain errors.
/// Use this when invalid SurrealQL should be a hard build failure.
///
/// # Panics
///
/// Panics if any `.surql` file fails to parse.
pub fn validate_schema_or_fail(dir: impl AsRef<Path>) {
	let errors = validate_schema(dir);
	if errors > 0 {
		panic!("SurrealQL validation failed: {errors} file(s) with errors");
	}
}

fn validate_schema_inner(dir: &Path, emit_warnings: bool) -> usize {
	println!("cargo:rerun-if-changed={}", dir.display());

	let mut error_count = 0;
	for entry in walkdir::WalkDir::new(dir).sort_by_file_name().into_iter() {
		let entry = match entry {
			Ok(e) => e,
			Err(e) => {
				println!(
					"cargo:warning=Skipping unreadable entry in {}: {e}",
					dir.display()
				);
				continue;
			}
		};
		let path = entry.path();
		if path.extension().is_some_and(|ext| ext == "surql") {
			println!("cargo:rerun-if-changed={}", path.display());
			let content = match std::fs::read_to_string(path) {
				Ok(c) => c,
				Err(e) => {
					if emit_warnings {
						println!("cargo:warning=Failed to read {}: {e}", path.display());
					}
					error_count += 1;
					continue;
				}
			};
			if let Err(e) = crate::parse(&content) {
				if emit_warnings {
					println!("cargo:warning={}: Invalid query: {e}", path.display());
				}
				error_count += 1;
			}
		}
	}

	if error_count > 0 && emit_warnings {
		println!("cargo:warning=SurrealQL validation: {error_count} file(s) with errors");
	}
	error_count
}

/// Generate typed Rust constants for SurrealQL functions defined in `.surql` files.
///
/// Walks `schema_dir` recursively, extracts all `DEFINE FUNCTION` statements,
/// and writes a Rust source file to `out_file` containing:
///
/// - A constant for each function name (e.g., `FN_GET_ENTITY: &str = "fn::get_entity"`)
/// - Doc comments with parameter types and return type
///
/// Emits `cargo:rerun-if-changed` for each schema file.
///
/// # Panics
///
/// Panics if any `.surql` file fails to parse, or if `out_file` cannot be written.
pub fn generate_typed_functions(schema_dir: impl AsRef<Path>, out_file: impl AsRef<Path>) {
	let schema_dir = schema_dir.as_ref();
	let out_file = out_file.as_ref();

	println!("cargo:rerun-if-changed={}", schema_dir.display());

	let mut defs = crate::SchemaDefinitions::default();
	for entry in walkdir::WalkDir::new(schema_dir)
		.sort_by_file_name()
		.into_iter()
	{
		let entry = match entry {
			Ok(e) => e,
			Err(e) => {
				println!(
					"cargo:warning=Skipping unreadable entry in {}: {e}",
					schema_dir.display()
				);
				continue;
			}
		};
		let path = entry.path();
		if path.extension().is_some_and(|ext| ext == "surql") {
			println!("cargo:rerun-if-changed={}", path.display());
			let content = match std::fs::read_to_string(path) {
				Ok(c) => c,
				Err(e) => {
					println!("cargo:warning=Failed to read {}: {e}", path.display());
					continue;
				}
			};
			let (stmts, _) = crate::parse_with_recovery(&content);
			match crate::extract_definitions_from_ast(&stmts) {
				Ok(file_defs) => {
					defs.functions.extend(file_defs.functions);
				}
				Err(e) => {
					println!(
						"cargo:warning=Failed to extract definitions from {}: {e}",
						path.display()
					);
				}
			}
		}
	}

	let mut code = String::new();
	code.push_str("// Auto-generated by surql-parser build helper.\n");
	code.push_str("// Do not edit manually.\n\n");

	let mut seen = std::collections::HashSet::new();
	for func in &defs.functions {
		let name = &func.name;
		if !seen.insert(name.clone()) {
			continue; // skip duplicate function definitions
		}

		// Build constant name: get_entity → FN_GET_ENTITY, ns::func → FN_NS_FUNC
		let const_name = name
			.chars()
			.map(|c| {
				if c == ':' {
					'_'
				} else {
					c.to_ascii_uppercase()
				}
			})
			.collect::<String>()
			// Collapse consecutive underscores from :: → __
			.replace("__", "_");

		// Build parameter documentation
		let params: Vec<String> = func
			.args
			.iter()
			.map(|(param_name, kind)| {
				let mut kind_str = String::new();
				kind.fmt_sql(&mut kind_str, SqlFormat::SingleLine);
				if param_name.starts_with('$') {
					format!("{param_name}: {kind_str}")
				} else {
					format!("${param_name}: {kind_str}")
				}
			})
			.collect();

		let returns_doc = func.returns.as_ref().map(|r| {
			let mut s = String::new();
			r.fmt_sql(&mut s, SqlFormat::SingleLine);
			s
		});

		// Write doc comment
		code.push_str(&format!("/// SurrealQL function: `fn::{name}`\n"));
		if !params.is_empty() {
			code.push_str(&format!("///\n/// Parameters: `{}`\n", params.join(", ")));
		}
		if let Some(ret) = &returns_doc {
			code.push_str(&format!("///\n/// Returns: `{ret}`\n"));
		}
		code.push_str(&format!(
			"pub const FN_{const_name}: &str = \"fn::{name}\";\n\n"
		));
	}

	if let Some(parent) = out_file.parent() {
		if let Err(e) = std::fs::create_dir_all(parent) {
			println!(
				"cargo:warning=Failed to create directory {}: {e}",
				parent.display()
			);
		}
	}
	std::fs::write(out_file, &code)
		.unwrap_or_else(|e| panic!("Failed to write {}: {e}", out_file.display()));
}
