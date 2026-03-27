//! SurrealQL linter — static analysis for common schema issues.
//!
//! Provides lint checks for `.surql` files including missing types,
//! schemaless tables, SELECT *, missing indexes, and unused functions.
//!
//! # Example
//!
//! ```
//! use surql_parser::lint::{lint_schema, LintSeverity};
//! use surql_parser::SchemaGraph;
//! use std::path::PathBuf;
//!
//! let source = "
//!     DEFINE TABLE user;
//!     DEFINE FIELD name ON user;
//!     SELECT * FROM user;
//! ";
//! let graph = SchemaGraph::from_source(source).unwrap();
//! let sources = vec![(PathBuf::from("schema.surql"), source.to_string())];
//! let results = lint_schema(&graph, &sources);
//! assert!(results.iter().any(|r| r.code == "schemaless-table"));
//! assert!(results.iter().any(|r| r.code == "missing-type"));
//! assert!(results.iter().any(|r| r.code == "select-star"));
//! ```

use std::collections::HashSet;
use std::path::PathBuf;

use crate::SchemaGraph;

/// Severity level for a lint result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LintSeverity {
	Warning,
	Info,
}

impl std::fmt::Display for LintSeverity {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			LintSeverity::Warning => write!(f, "warn"),
			LintSeverity::Info => write!(f, "info"),
		}
	}
}

/// A single lint finding.
#[derive(Debug, Clone)]
pub struct LintResult {
	pub file: String,
	pub line: u32,
	pub col: u32,
	pub code: String,
	pub message: String,
	pub severity: LintSeverity,
}

impl std::fmt::Display for LintResult {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}:{}:{} [{}] {}",
			self.file, self.line, self.col, self.code, self.message
		)
	}
}

/// Run all lint checks against a schema graph and its source files.
///
/// `sources` is a list of (file_path, file_content) pairs so we can report
/// accurate line/column positions and detect SELECT * patterns.
pub fn lint_schema(schema: &SchemaGraph, sources: &[(PathBuf, String)]) -> Vec<LintResult> {
	let mut results = Vec::new();

	lint_missing_type(schema, sources, &mut results);
	lint_schemaless_table(schema, sources, &mut results);
	lint_select_star(sources, &mut results);
	lint_missing_index(schema, sources, &mut results);
	lint_unused_function(schema, sources, &mut results);

	results.sort_by(|a, b| {
		a.file
			.cmp(&b.file)
			.then_with(|| a.line.cmp(&b.line))
			.then_with(|| a.col.cmp(&b.col))
	});

	results
}

/// Apply auto-fixes for fixable lints. Returns the fixed source content
/// and the number of fixes applied.
///
/// Currently fixable: `missing-type` (adds `TYPE any`).
pub fn apply_fixes(source: &str) -> (String, u32) {
	let mut fixed = String::new();
	let mut fix_count = 0u32;

	for line in source.lines() {
		let trimmed = line.trim();
		let upper = trimmed.to_uppercase();

		// Only auto-fix single-line DEFINE FIELD statements (entire statement on one line
		// ending with semicolon). Multi-line DEFINE FIELD spans are skipped to avoid corruption.
		if upper.starts_with("DEFINE FIELD")
			&& trimmed.ends_with(';')
			&& !upper.contains(" TYPE ")
			&& !upper.contains(" FLEXIBLE ")
		{
			let semicolon_stripped = trimmed.strip_suffix(';').unwrap_or(trimmed);
			let leading_ws: &str = &line[..line.len() - line.trim_start().len()];
			fixed.push_str(leading_ws);
			fixed.push_str(semicolon_stripped);
			fixed.push_str(" TYPE any;");
			fixed.push('\n');
			fix_count += 1;
		} else {
			fixed.push_str(line);
			fixed.push('\n');
		}
	}

	if source.ends_with('\n') || fixed.is_empty() {
		// already has trailing newline
	} else {
		// remove extra trailing newline we added
		fixed.pop();
	}

	(fixed, fix_count)
}

// ─── Individual Lints ───

/// Warn on DEFINE FIELD without TYPE annotation.
fn lint_missing_type(
	schema: &SchemaGraph,
	sources: &[(PathBuf, String)],
	results: &mut Vec<LintResult>,
) {
	for table_name in schema.table_names() {
		for field in schema.fields_of(table_name) {
			if field.kind.is_none() {
				let Some((file, line, col)) = find_field_location(sources, table_name, &field.name)
				else {
					continue;
				};
				results.push(LintResult {
					file,
					line,
					col,
					code: "missing-type".into(),
					message: format!(
						"DEFINE FIELD {} ON {} \u{2014} no TYPE specified",
						field.name, table_name
					),
					severity: LintSeverity::Warning,
				});
			}
		}
	}
}

/// Warn on DEFINE TABLE without SCHEMAFULL.
fn lint_schemaless_table(
	schema: &SchemaGraph,
	sources: &[(PathBuf, String)],
	results: &mut Vec<LintResult>,
) {
	for table_name in schema.table_names() {
		if let Some(table) = schema.table(table_name)
			&& !table.full
		{
			let Some((file, line, col)) = find_define_table_location(sources, table_name) else {
				continue;
			};
			results.push(LintResult {
				file,
				line,
				col,
				code: "schemaless-table".into(),
				message: format!("DEFINE TABLE {} \u{2014} consider SCHEMAFULL", table_name),
				severity: LintSeverity::Warning,
			});
		}
	}
}

/// Warn on SELECT * usage.
fn lint_select_star(sources: &[(PathBuf, String)], results: &mut Vec<LintResult>) {
	for (path, content) in sources {
		let file_str = path.display().to_string();
		for (line_num, line) in content.lines().enumerate() {
			let trimmed = line.trim();
			if trimmed.starts_with("--") || trimmed.starts_with("//") {
				continue;
			}
			let upper = line.to_uppercase();
			if let Some(pos) = upper.find("SELECT *") {
				// Skip if SELECT * appears after a comment marker or inside a string
				let before_select = &line[..pos];
				if before_select.contains("--")
					|| before_select.contains("//")
					|| is_inside_string(before_select)
				{
					continue;
				}
				let after = &upper[pos + 8..];
				if after.trim_start().starts_with("FROM")
					|| after.trim_start().starts_with(',')
					|| after.trim_start().is_empty()
				{
					results.push(LintResult {
						file: file_str.clone(),
						line: (line_num + 1) as u32,
						col: (pos + 1) as u32,
						code: "select-star".into(),
						message: format!(
							"{} \u{2014} specify fields for production",
							trimmed.trim_end_matches(';')
						),
						severity: LintSeverity::Info,
					});
				}
			}
		}
	}
}

/// Check if position is likely inside a string literal by counting unescaped quotes.
/// An odd number of single or double quotes before a position means we are inside a string.
fn is_inside_string(before: &str) -> bool {
	let single_quotes = before.chars().filter(|&c| c == '\'').count();
	let double_quotes = before.chars().filter(|&c| c == '"').count();
	single_quotes % 2 != 0 || double_quotes % 2 != 0
}

/// Warn on tables with 5+ fields but no indexes.
fn lint_missing_index(
	schema: &SchemaGraph,
	sources: &[(PathBuf, String)],
	results: &mut Vec<LintResult>,
) {
	for table_name in schema.table_names() {
		let fields = schema.fields_of(table_name);
		let indexes = schema.indexes_of(table_name);
		if fields.len() >= 5 && indexes.is_empty() {
			let Some((file, line, col)) = find_define_table_location(sources, table_name) else {
				continue;
			};
			results.push(LintResult {
				file,
				line,
				col,
				code: "missing-index".into(),
				message: format!(
					"DEFINE TABLE {} has {} fields but no indexes",
					table_name,
					fields.len()
				),
				severity: LintSeverity::Info,
			});
		}
	}
}

/// Warn on DEFINE FUNCTION not called anywhere in the project.
fn lint_unused_function(
	schema: &SchemaGraph,
	sources: &[(PathBuf, String)],
	results: &mut Vec<LintResult>,
) {
	let all_content: String = sources
		.iter()
		.map(|(_, c)| c.as_str())
		.collect::<Vec<_>>()
		.join("\n");
	let fn_names: Vec<&str> = schema.function_names().collect();

	let mut called: HashSet<&str> = HashSet::new();
	for name in &fn_names {
		// Match fn::name only at word boundaries: followed by '(', ' ', ';', ')', ',', or EOL
		let ref_pattern = format!("fn::{name}");
		for line in all_content.lines() {
			let trimmed = line.trim();
			if trimmed.to_uppercase().starts_with("DEFINE FUNCTION") {
				continue;
			}
			// Skip comment lines
			if trimmed.starts_with("--") || trimmed.starts_with("//") {
				continue;
			}
			if let Some(pos) = line.find(&ref_pattern) {
				let after_pos = pos + ref_pattern.len();
				let next_char = line[after_pos..].chars().next();
				// Require word boundary after the function name
				let at_boundary = match next_char {
					None => true,
					Some(c) => !c.is_alphanumeric() && c != '_' && c != ':',
				};
				if at_boundary {
					called.insert(name);
					break;
				}
			}
		}
	}

	for name in &fn_names {
		if !called.contains(name) {
			let Some((file, line, col)) = find_define_function_location(sources, name) else {
				continue;
			};
			results.push(LintResult {
				file,
				line,
				col,
				code: "unused-function".into(),
				message: format!("fn::{name} is defined but never called in the project",),
				severity: LintSeverity::Info,
			});
		}
	}
}

// ─── Location Finding ───

fn find_field_location(
	sources: &[(PathBuf, String)],
	table_name: &str,
	field_name: &str,
) -> Option<(String, u32, u32)> {
	let pattern_upper = format!("DEFINE FIELD {} ON {}", field_name, table_name).to_uppercase();
	let pattern_upper_table =
		format!("DEFINE FIELD {} ON TABLE {}", field_name, table_name).to_uppercase();

	for (path, content) in sources {
		for (line_num, line) in content.lines().enumerate() {
			let upper = line.to_uppercase();
			if upper.contains(&pattern_upper) || upper.contains(&pattern_upper_table) {
				let col = upper.find("DEFINE FIELD").map(|p| p + 1).unwrap_or(1);
				return Some((
					path.display().to_string(),
					(line_num + 1) as u32,
					col as u32,
				));
			}
		}
	}
	None
}

fn find_define_table_location(
	sources: &[(PathBuf, String)],
	table_name: &str,
) -> Option<(String, u32, u32)> {
	let pattern_upper = format!("DEFINE TABLE {}", table_name).to_uppercase();

	for (path, content) in sources {
		for (line_num, line) in content.lines().enumerate() {
			let upper = line.to_uppercase();
			if upper.contains(&pattern_upper) {
				let col = upper.find("DEFINE TABLE").map(|p| p + 1).unwrap_or(1);
				return Some((
					path.display().to_string(),
					(line_num + 1) as u32,
					col as u32,
				));
			}
		}
	}
	None
}

fn find_define_function_location(
	sources: &[(PathBuf, String)],
	fn_name: &str,
) -> Option<(String, u32, u32)> {
	let pattern = format!("fn::{fn_name}");

	for (path, content) in sources {
		for (line_num, line) in content.lines().enumerate() {
			let upper = line.to_uppercase();
			if upper.contains("DEFINE FUNCTION") && line.contains(&pattern) {
				let col = line.find("DEFINE").map(|p| p + 1).unwrap_or(1);
				return Some((
					path.display().to_string(),
					(line_num + 1) as u32,
					col as u32,
				));
			}
		}
	}
	None
}

#[cfg(test)]
mod tests {
	use super::*;

	fn lint_source(source: &str) -> Vec<LintResult> {
		let graph = SchemaGraph::from_source(source).unwrap();
		let sources = vec![(PathBuf::from("schema.surql"), source.to_string())];
		lint_schema(&graph, &sources)
	}

	#[test]
	fn should_detect_missing_type_on_field() {
		let results = lint_source("DEFINE TABLE user SCHEMAFULL;\nDEFINE FIELD name ON user;\n");
		let missing = results.iter().filter(|r| r.code == "missing-type").count();
		assert_eq!(missing, 1, "expected 1 missing-type lint, got: {results:?}");
	}

	#[test]
	fn should_not_flag_field_with_type() {
		let results =
			lint_source("DEFINE TABLE user SCHEMAFULL;\nDEFINE FIELD name ON user TYPE string;\n");
		let missing = results.iter().filter(|r| r.code == "missing-type").count();
		assert_eq!(
			missing, 0,
			"field with TYPE should not be flagged: {results:?}"
		);
	}

	#[test]
	fn should_detect_schemaless_table() {
		let results = lint_source("DEFINE TABLE post;\n");
		let schemaless = results
			.iter()
			.filter(|r| r.code == "schemaless-table")
			.count();
		assert_eq!(
			schemaless, 1,
			"expected 1 schemaless-table lint: {results:?}"
		);
	}

	#[test]
	fn should_not_flag_schemafull_table() {
		let results = lint_source("DEFINE TABLE post SCHEMAFULL;\n");
		let schemaless = results
			.iter()
			.filter(|r| r.code == "schemaless-table")
			.count();
		assert_eq!(
			schemaless, 0,
			"SCHEMAFULL table should not be flagged: {results:?}"
		);
	}

	#[test]
	fn should_detect_select_star() {
		let results = lint_source("DEFINE TABLE user SCHEMAFULL;\nSELECT * FROM user;\n");
		let stars = results.iter().filter(|r| r.code == "select-star").count();
		assert_eq!(stars, 1, "expected 1 select-star lint: {results:?}");
	}

	#[test]
	fn should_not_flag_explicit_select() {
		let results = lint_source("DEFINE TABLE user SCHEMAFULL;\nSELECT name, age FROM user;\n");
		let stars = results.iter().filter(|r| r.code == "select-star").count();
		assert_eq!(
			stars, 0,
			"explicit SELECT should not be flagged: {results:?}"
		);
	}

	#[test]
	fn should_detect_missing_index_with_many_fields() {
		let source = "\
DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD name ON user TYPE string;
DEFINE FIELD email ON user TYPE string;
DEFINE FIELD age ON user TYPE int;
DEFINE FIELD bio ON user TYPE string;
DEFINE FIELD avatar ON user TYPE string;
";
		let results = lint_source(source);
		let missing_idx = results.iter().filter(|r| r.code == "missing-index").count();
		assert_eq!(
			missing_idx, 1,
			"expected 1 missing-index lint for 5 fields with no index: {results:?}"
		);
	}

	#[test]
	fn should_not_flag_table_with_index() {
		let source = "\
DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD name ON user TYPE string;
DEFINE FIELD email ON user TYPE string;
DEFINE FIELD age ON user TYPE int;
DEFINE FIELD bio ON user TYPE string;
DEFINE FIELD avatar ON user TYPE string;
DEFINE INDEX email_idx ON user FIELDS email UNIQUE;
";
		let results = lint_source(source);
		let missing_idx = results.iter().filter(|r| r.code == "missing-index").count();
		assert_eq!(
			missing_idx, 0,
			"table with index should not be flagged: {results:?}"
		);
	}

	#[test]
	fn should_detect_unused_function() {
		let source = "\
DEFINE TABLE user SCHEMAFULL;
DEFINE FUNCTION fn::greet($name: string) { RETURN 'Hello, ' + $name; };
";
		let results = lint_source(source);
		let unused = results
			.iter()
			.filter(|r| r.code == "unused-function")
			.count();
		assert_eq!(unused, 1, "expected 1 unused-function lint: {results:?}");
	}

	#[test]
	fn should_not_flag_called_function() {
		let source = "\
DEFINE TABLE user SCHEMAFULL;
DEFINE FUNCTION fn::greet($name: string) { RETURN 'Hello, ' + $name; };
";
		let graph = SchemaGraph::from_source(source).unwrap();
		let caller = "LET $greeting = fn::greet('World');";
		let sources = vec![
			(PathBuf::from("schema.surql"), source.to_string()),
			(PathBuf::from("queries.surql"), caller.to_string()),
		];
		let results = lint_schema(&graph, &sources);
		let unused = results
			.iter()
			.filter(|r| r.code == "unused-function")
			.count();
		assert_eq!(
			unused, 0,
			"called function should not be flagged: {results:?}"
		);
	}

	#[test]
	fn should_apply_missing_type_fix() {
		let source = "DEFINE FIELD name ON user;\nDEFINE FIELD age ON user TYPE int;\n";
		let (fixed, count) = apply_fixes(source);
		assert_eq!(count, 1);
		assert!(fixed.contains("DEFINE FIELD name ON user TYPE any;"));
		assert!(fixed.contains("DEFINE FIELD age ON user TYPE int;"));
	}

	#[test]
	fn should_not_fix_field_with_type() {
		let source = "DEFINE FIELD name ON user TYPE string;\n";
		let (fixed, count) = apply_fixes(source);
		assert_eq!(count, 0);
		assert_eq!(fixed, source);
	}

	#[test]
	fn should_skip_comment_lines_for_select_star() {
		let results = lint_source("-- SELECT * FROM user;\n");
		let stars = results.iter().filter(|r| r.code == "select-star").count();
		assert_eq!(stars, 0, "comments should not trigger select-star lint");
	}

	#[test]
	fn should_skip_select_star_inside_string_literal() {
		let source = "DEFINE TABLE user SCHEMAFULL;\nLET $q = 'SELECT * FROM user';\n";
		let graph = SchemaGraph::from_source(source).unwrap();
		let sources = vec![(PathBuf::from("schema.surql"), source.to_string())];
		let results = lint_schema(&graph, &sources);
		let stars = results.iter().filter(|r| r.code == "select-star").count();
		assert_eq!(
			stars, 0,
			"SELECT * inside string literal should not be flagged"
		);
	}

	#[test]
	fn should_skip_select_star_after_inline_comment() {
		let source = "DEFINE TABLE user SCHEMAFULL;\nLET $x = 1; -- SELECT * FROM user;\n";
		let graph = SchemaGraph::from_source(source).unwrap();
		let sources = vec![(PathBuf::from("schema.surql"), source.to_string())];
		let results = lint_schema(&graph, &sources);
		let stars = results.iter().filter(|r| r.code == "select-star").count();
		assert_eq!(
			stars, 0,
			"SELECT * after inline comment should not be flagged"
		);
	}

	#[test]
	fn should_not_fix_multiline_define_field() {
		let source = "DEFINE FIELD name ON user\n\tDEFAULT 'test';\n";
		let (fixed, count) = apply_fixes(source);
		assert_eq!(count, 0, "multi-line DEFINE FIELD should not be auto-fixed");
		assert_eq!(fixed, source);
	}

	#[test]
	fn should_not_flag_fn_name_prefix_as_used() {
		let source = "\
DEFINE TABLE user SCHEMAFULL;
DEFINE FUNCTION fn::get($id: string) { RETURN 1; };
DEFINE FUNCTION fn::get_all() { RETURN 2; };
LET $x = fn::get_all();
";
		let graph = SchemaGraph::from_source(source).unwrap();
		let sources = vec![(PathBuf::from("schema.surql"), source.to_string())];
		let results = lint_schema(&graph, &sources);
		let unused: Vec<&str> = results
			.iter()
			.filter(|r| r.code == "unused-function")
			.map(|r| r.message.as_str())
			.collect();
		assert!(
			unused.iter().any(|m| m.contains("fn::get ")),
			"fn::get should be flagged as unused (fn::get_all is called, not fn::get): {unused:?}"
		);
	}
}
