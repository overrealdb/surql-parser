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
