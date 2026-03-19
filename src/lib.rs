//! surql-parser — Standalone SurrealQL parser extracted from SurrealDB.
//!
//! Provides a complete SurrealQL parser without depending on the SurrealDB engine.
//! Useful for building tools, linters, formatters, IDE extensions, and migration systems.
//!
//! # Quick Start
//!
//! ```
//! let ast = surql_parser::parse("SELECT name, age FROM user WHERE age > 18").unwrap();
//! assert!(!ast.expressions.is_empty());
//! ```
//!
//! # Sync with SurrealDB
//!
//! Parser source is auto-extracted from SurrealDB via `tools/transform/`.
//! See UPSTREAM_SYNC.md for details.

#[macro_use]
extern crate tracing;

pub mod compat;
pub mod config;

#[cfg(feature = "build")]
pub mod build;

#[allow(
	clippy::useless_conversion,
	clippy::large_enum_variant,
	clippy::match_single_binding,
	clippy::needless_borrow
)]
pub mod upstream;

// ─── Public API ───

/// Parse a SurrealQL query string into an AST.
///
/// Returns a list of top-level expressions (statements).
///
/// # Example
///
/// ```
/// let ast = surql_parser::parse("CREATE user SET name = 'Alice'").unwrap();
/// assert_eq!(ast.expressions.len(), 1);
/// ```
pub fn parse(input: &str) -> anyhow::Result<Ast> {
	upstream::syn::parse(input)
}

/// Parse a SurrealQL query with custom parser settings.
pub fn parse_with_settings(input: &str, settings: ParserSettings) -> anyhow::Result<Ast> {
	upstream::syn::parse_with_settings(input.as_bytes(), settings, async |parser, stk| {
		parser.parse_query(stk).await
	})
}

/// Parse a SurrealQL type annotation (e.g., `record<user>`, `option<string>`).
pub fn parse_kind(input: &str) -> anyhow::Result<Kind> {
	upstream::syn::kind(input)
}

/// Check if a string could be a reserved keyword in certain contexts.
pub fn is_reserved_keyword(s: &str) -> bool {
	upstream::syn::could_be_reserved_keyword(s)
}

// ─── Schema Extraction ───

/// All definitions found in a SurrealQL file.
///
/// Use `extract_definitions()` to get this from a .surql file.
/// This is the primary tool for migration systems and schema analyzers.
use upstream::sql::statements::define;

#[derive(Debug, Default)]
pub struct SchemaDefinitions {
	pub namespaces: Vec<statements::DefineNamespaceStatement>,
	pub databases: Vec<define::DefineDatabaseStatement>,
	pub tables: Vec<statements::DefineTableStatement>,
	pub fields: Vec<statements::DefineFieldStatement>,
	pub indexes: Vec<statements::DefineIndexStatement>,
	pub functions: Vec<statements::DefineFunctionStatement>,
	pub analyzers: Vec<define::DefineAnalyzerStatement>,
	pub events: Vec<statements::DefineEventStatement>,
	pub params: Vec<define::DefineParamStatement>,
	pub users: Vec<define::DefineUserStatement>,
	pub accesses: Vec<define::DefineAccessStatement>,
}

/// Extract all DEFINE statements from a SurrealQL string.
///
/// Useful for schema analysis, migration tools, and validation.
///
/// # Example
///
/// ```
/// let defs = surql_parser::extract_definitions("
///     DEFINE TABLE user SCHEMAFULL;
///     DEFINE FIELD name ON user TYPE string;
///     DEFINE FIELD age ON user TYPE int DEFAULT 0;
///     DEFINE INDEX email_idx ON user FIELDS email UNIQUE;
///     DEFINE FUNCTION fn::greet($name: string) { RETURN 'Hello, ' + $name; };
/// ").unwrap();
///
/// assert_eq!(defs.tables.len(), 1);
/// assert_eq!(defs.fields.len(), 2);
/// assert_eq!(defs.indexes.len(), 1);
/// assert_eq!(defs.functions.len(), 1);
/// ```
pub fn extract_definitions(input: &str) -> anyhow::Result<SchemaDefinitions> {
	let ast = parse(input)?;
	let mut defs = SchemaDefinitions::default();

	for top in &ast.expressions {
		if let upstream::sql::ast::TopLevelExpr::Expr(Expr::Define(stmt)) = top {
			use define::DefineStatement as DS;
			match stmt.as_ref() {
				DS::Namespace(s) => defs.namespaces.push(s.clone()),
				DS::Database(s) => defs.databases.push(s.clone()),
				DS::Table(s) => defs.tables.push(s.clone()),
				DS::Field(s) => defs.fields.push(s.clone()),
				DS::Index(s) => defs.indexes.push(s.clone()),
				DS::Function(s) => defs.functions.push(s.clone()),
				DS::Analyzer(s) => defs.analyzers.push(s.clone()),
				DS::Event(s) => defs.events.push(s.clone()),
				DS::Param(s) => defs.params.push(s.clone()),
				DS::User(s) => defs.users.push(s.clone()),
				DS::Access(s) => defs.accesses.push(s.clone()),
				_ => {} // Config, Api, Bucket, Sequence, Module, Model — less common
			}
		}
	}

	Ok(defs)
}

/// List all function names defined in a SurrealQL string.
///
/// # Example
///
/// ```
/// let fns = surql_parser::list_functions("
///     DEFINE FUNCTION fn::greet($name: string) { RETURN 'Hello, ' + $name; };
///     DEFINE FUNCTION fn::add($a: int, $b: int) { RETURN $a + $b; };
/// ").unwrap();
///
/// assert_eq!(fns, vec!["greet", "add"]);
/// ```
pub fn list_functions(input: &str) -> anyhow::Result<Vec<String>> {
	let defs = extract_definitions(input)?;
	Ok(defs
		.functions
		.iter()
		.map(|f| {
			use surrealdb_types::{SqlFormat, ToSql};
			let mut name = String::new();
			f.name.fmt_sql(&mut name, SqlFormat::SingleLine);
			name
		})
		.collect())
}

/// List all table names defined in a SurrealQL string.
///
/// # Example
///
/// ```
/// let tables = surql_parser::list_tables("
///     DEFINE TABLE user SCHEMAFULL;
///     DEFINE TABLE post SCHEMALESS;
///     SELECT * FROM user;
/// ").unwrap();
///
/// assert_eq!(tables, vec!["user", "post"]);
/// ```
pub fn list_tables(input: &str) -> anyhow::Result<Vec<String>> {
	let defs = extract_definitions(input)?;
	Ok(defs
		.tables
		.iter()
		.map(|t| {
			use surrealdb_types::{SqlFormat, ToSql};
			let mut name = String::new();
			t.name.fmt_sql(&mut name, SqlFormat::SingleLine);
			name
		})
		.collect())
}

/// Format an AST back to SurrealQL string.
///
/// # Example
///
/// ```
/// let ast = surql_parser::parse("SELECT * FROM user").unwrap();
/// let sql = surql_parser::format(&ast);
/// assert!(sql.contains("SELECT"));
/// ```
pub fn format(ast: &Ast) -> String {
	use surrealdb_types::{SqlFormat, ToSql};
	let mut buf = String::new();
	ast.fmt_sql(&mut buf, SqlFormat::SingleLine);
	buf
}

// ─── Parameter Extraction ───

/// Extract all `$param` names used in a SurrealQL query.
///
/// Parses the input, then scans for parameter tokens. Returns a sorted,
/// deduplicated list of parameter names (without the `$` prefix).
///
/// Parameters inside `DEFINE FUNCTION` signatures are excluded —
/// only "free" parameters (query-level bindings) are returned.
///
/// # Example
///
/// ```
/// let params = surql_parser::extract_params(
///     "SELECT * FROM user WHERE age > $min AND name = $name"
/// ).unwrap();
/// assert_eq!(params, vec!["min", "name"]);
/// ```
pub fn extract_params(input: &str) -> anyhow::Result<Vec<String>> {
	// Validate syntax first
	parse(input)?;
	// Extract params via lexer-level scan (handles strings, comments correctly)
	Ok(scan_params(input))
}

/// Scan a (known-valid) SurrealQL string for `$param` tokens.
///
/// Skips string literals and comments. Returns sorted, deduplicated names.
fn scan_params(input: &str) -> Vec<String> {
	let mut params = std::collections::BTreeSet::new();
	let bytes = input.as_bytes();
	let len = bytes.len();
	let mut i = 0;

	while i < len {
		match bytes[i] {
			// Skip single-quoted strings: 'text''s escaped'
			b'\'' => {
				i += 1;
				while i < len {
					if bytes[i] == b'\'' {
						i += 1;
						if i < len && bytes[i] == b'\'' {
							i += 1; // escaped ''
							continue;
						}
						break;
					}
					i += 1;
				}
			}
			// Skip double-quoted strings: "text\"s escaped"
			b'"' => {
				i += 1;
				while i < len {
					if bytes[i] == b'\\' {
						i += 2;
						continue;
					}
					if bytes[i] == b'"' {
						i += 1;
						break;
					}
					i += 1;
				}
			}
			// Skip backtick-quoted identifiers: `field name`
			b'`' => {
				i += 1;
				while i < len {
					if bytes[i] == b'`' {
						i += 1;
						break;
					}
					i += 1;
				}
			}
			// Skip line comments: -- ...
			b'-' if i + 1 < len && bytes[i + 1] == b'-' => {
				i += 2;
				while i < len && bytes[i] != b'\n' {
					i += 1;
				}
			}
			// Skip block comments: /* ... */
			b'/' if i + 1 < len && bytes[i + 1] == b'*' => {
				i += 2;
				let mut depth = 1u32;
				while i + 1 < len && depth > 0 {
					if bytes[i] == b'/' && bytes[i + 1] == b'*' {
						depth += 1;
						i += 2;
					} else if bytes[i] == b'*' && bytes[i + 1] == b'/' {
						depth -= 1;
						i += 2;
					} else {
						i += 1;
					}
				}
			}
			// Collect $param
			b'$' => {
				i += 1;
				let start = i;
				while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
					i += 1;
				}
				if i > start {
					let name = &input[start..i];
					params.insert(name.to_string());
				}
			}
			_ => i += 1,
		}
	}

	params.into_iter().collect()
}

// ─── Re-exports ───

/// The parsed AST (list of top-level statements).
pub use upstream::sql::Ast;

/// A single expression in the AST.
pub use upstream::sql::expression::Expr;

/// Parser configuration settings.
pub use upstream::syn::ParserSettings;

/// SurrealQL type annotation (e.g., `string`, `record<user>`, `array<int>`).
pub use upstream::sql::Kind;

/// An identifier path (e.g., `user.name`, `->knows->person`).
pub use upstream::sql::Idiom;

/// A SurrealQL statement (SELECT, CREATE, DEFINE, etc.).
pub use upstream::sql::statements;

/// Syntax error type.
pub use upstream::syn::error;
