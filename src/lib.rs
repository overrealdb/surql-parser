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
pub mod error;

#[cfg(feature = "build")]
pub mod build;

#[allow(
	clippy::useless_conversion,
	clippy::large_enum_variant,
	clippy::match_single_binding,
	clippy::needless_borrow
)]
pub mod upstream;

pub mod builtins_generated;
pub mod diff;
pub mod doc_urls;
pub mod filesystem;
pub mod formatting;
pub mod keywords;
pub mod lint;
pub mod params;
pub mod recovery;
pub mod schema_graph;
pub mod schema_lookup;

// Re-export for backward compat
pub use error::{Error, Result};
pub use filesystem::*;
pub use keywords::all_keywords;
pub use params::*;
pub use schema_lookup::*;

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
pub fn parse(input: &str) -> Result<Ast> {
	upstream::syn::parse(input).map_err(|e| Error::Parse(e.to_string()))
}

/// Parse a SurrealQL query with custom parser settings.
pub fn parse_with_settings(input: &str, settings: ParserSettings) -> Result<Ast> {
	upstream::syn::parse_with_settings(input.as_bytes(), settings, async |parser, stk| {
		parser.parse_query(stk).await
	})
	.map_err(|e| Error::Parse(e.to_string()))
}

/// Parse a SurrealQL type annotation (e.g., `record<user>`, `option<string>`).
pub fn parse_kind(input: &str) -> Result<Kind> {
	upstream::syn::kind(input).map_err(|e| Error::Parse(e.to_string()))
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
	/// Current NS/DB context from USE statements (tracked during extraction)
	pub current_ns: Option<String>,
	pub current_db: Option<String>,
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
pub fn extract_definitions(input: &str) -> Result<SchemaDefinitions> {
	let ast = parse(input)?;
	extract_definitions_from_ast(&ast.expressions)
}

/// Extract definitions from pre-parsed statements (e.g., from error-recovering parser).
pub fn extract_definitions_from_ast(
	stmts: &[upstream::sql::ast::TopLevelExpr],
) -> Result<SchemaDefinitions> {
	let mut defs = SchemaDefinitions::default();

	for top in stmts {
		// Track USE NS/DB context changes
		if let upstream::sql::ast::TopLevelExpr::Use(use_stmt) = top {
			let (ns, db) = extract_use_context(use_stmt);
			if let Some(ns) = ns {
				defs.current_ns = Some(ns);
			}
			if let Some(db) = db {
				defs.current_db = Some(db);
			}
			continue;
		}

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
				_ => {}
			}
		}
	}

	Ok(defs)
}

fn extract_use_context(
	use_stmt: &upstream::sql::statements::r#use::UseStatement,
) -> (Option<String>, Option<String>) {
	use surrealdb_types::{SqlFormat, ToSql};
	use upstream::sql::statements::r#use::UseStatement;

	fn expr_to_string(expr: &Expr) -> String {
		let mut s = String::new();
		expr.fmt_sql(&mut s, SqlFormat::SingleLine);
		s
	}

	match use_stmt {
		UseStatement::Ns(ns) => (Some(expr_to_string(ns)), None),
		UseStatement::Db(db) => (None, Some(expr_to_string(db))),
		UseStatement::NsDb(ns, db) => (Some(expr_to_string(ns)), Some(expr_to_string(db))),
		UseStatement::Default => (None, None),
	}
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
pub fn list_functions(input: &str) -> Result<Vec<String>> {
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
pub fn list_tables(input: &str) -> Result<Vec<String>> {
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

// ─── Diagnostics ───

/// A parse diagnostic with source location.
#[derive(Debug, Clone)]
pub struct ParseDiagnostic {
	pub message: String,
	/// 1-indexed line number.
	pub line: usize,
	/// 1-indexed column number (in chars).
	pub column: usize,
	/// 1-indexed end line number.
	pub end_line: usize,
	/// 1-indexed end column number.
	pub end_column: usize,
}

/// Parse SurrealQL and return structured diagnostics on error.
///
/// Unlike [`parse()`], this function returns diagnostics with precise
/// source positions suitable for LSP and IDE integration.
///
/// # Example
///
/// ```
/// let result = surql_parser::parse_for_diagnostics("SELEC * FROM user");
/// assert!(result.is_err());
/// let diags = result.unwrap_err();
/// assert!(!diags.is_empty());
/// assert_eq!(diags[0].line, 1);
/// ```
pub fn parse_for_diagnostics(input: &str) -> std::result::Result<Ast, Vec<ParseDiagnostic>> {
	use upstream::syn::error::Location;
	use upstream::syn::token::Span;

	let bytes = input.as_bytes();
	if bytes.len() > u32::MAX as usize {
		return Err(vec![ParseDiagnostic {
			message: "Query too large".into(),
			line: 1,
			column: 1,
			end_line: 1,
			end_column: 1,
		}]);
	}

	let settings = upstream::syn::settings_from_capabilities(&compat::Capabilities::all());
	let mut parser = upstream::syn::parser::Parser::new_with_settings(bytes, settings);
	let mut stack = reblessive::Stack::new();

	match stack.enter(|stk| parser.parse_query(stk)).finish() {
		Ok(ast) => Ok(ast),
		Err(syntax_error) => {
			// Collect spans from the error chain
			let mut spans: Vec<Span> = Vec::new();
			let err = syntax_error.update_spans(|span| {
				spans.push(*span);
			});

			// Get rendered error messages
			let rendered = err.render_on(input);

			let message = rendered.errors.join(": ");

			let mut diags: Vec<ParseDiagnostic> = spans
				.iter()
				.map(|span| {
					let range = Location::range_of_span(input, *span);
					ParseDiagnostic {
						message: message.clone(),
						line: range.start.line,
						column: range.start.column,
						end_line: range.end.line,
						end_column: range.end.column,
					}
				})
				.collect();

			// If no spans but there are error messages, create a fallback diagnostic
			if diags.is_empty() && !message.is_empty() {
				diags.push(ParseDiagnostic {
					message,
					line: 1,
					column: 1,
					end_line: 1,
					end_column: 1,
				});
			}

			Err(diags)
		}
	}
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

/// Syntax error type from the upstream parser.
pub use upstream::syn::error as syntax_error;

pub use recovery::parse_with_recovery;
pub use schema_graph::{DependencyNode, SchemaGraph};

// ─── Built-in Function Lookup ───

/// Look up a built-in SurrealQL function by its full name (e.g., `"string::len"`).
pub fn builtin_function(name: &str) -> Option<&'static builtins_generated::BuiltinFn> {
	use std::collections::HashMap;
	use std::sync::LazyLock;

	static INDEX: LazyLock<HashMap<&'static str, &'static builtins_generated::BuiltinFn>> =
		LazyLock::new(|| {
			builtins_generated::BUILTINS
				.iter()
				.map(|f| (f.name, f))
				.collect()
		});

	INDEX.get(name).copied()
}

/// Return all built-in functions in a given namespace (e.g., `"string"` returns all `string::*`).
pub fn builtins_in_namespace(ns: &str) -> Vec<&'static builtins_generated::BuiltinFn> {
	let prefix = format!("{ns}::");
	builtins_generated::BUILTINS
		.iter()
		.filter(|f| f.name.starts_with(&prefix))
		.collect()
}
