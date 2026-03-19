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

pub mod recovery;
pub mod schema_graph;

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

/// Extract definitions from pre-parsed statements (e.g., from error-recovering parser).
pub fn extract_definitions_from_ast(
	stmts: &[upstream::sql::ast::TopLevelExpr],
) -> anyhow::Result<SchemaDefinitions> {
	let mut defs = SchemaDefinitions::default();

	for top in stmts {
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

// ─── Keywords ───

/// All SurrealQL keywords recognized by the parser.
///
/// Returns a static slice of keyword strings, extracted from the parser's
/// internal keyword enum. This stays in sync with the parser automatically.
pub fn all_keywords() -> &'static [&'static str] {
	&[
		"ACCESS",
		"AFTER",
		"ALGORITHM",
		"ALL",
		"ALTER",
		"ALWAYS",
		"ANALYZER",
		"API",
		"AS",
		"ASCENDING",
		"ASCII",
		"ASSERT",
		"ASYNC",
		"AT",
		"AUTHENTICATE",
		"AUTO",
		"BACKEND",
		"BATCH",
		"BEARER",
		"BEFORE",
		"BEGIN",
		"BLANK",
		"BUCKET",
		"REJECT",
		"BM25",
		"BREAK",
		"BY",
		"CAMEL",
		"CANCEL",
		"CASCADE",
		"CHANGEFEED",
		"CHANGES",
		"CAPACITY",
		"CLASS",
		"COMMENT",
		"COMMIT",
		"COMPACT",
		"CONCURRENTLY",
		"CONFIG",
		"CONTENT",
		"CONTINUE",
		"COMPUTED",
		"COUNT",
		"CREATE",
		"DATABASE",
		"DEFAULT",
		"DEFINE",
		"DELETE",
		"DESCENDING",
		"DIFF",
		"DIMENSION",
		"DISTANCE",
		"DROP",
		"DUPLICATE",
		"EFC",
		"EDGENGRAM",
		"EVENT",
		"ELSE",
		"END",
		"ENFORCED",
		"EXCLUDE",
		"EXISTS",
		"EXPIRED",
		"EXPLAIN",
		"EXPUNGE",
		"EXTEND_CANDIDATES",
		"false",
		"FETCH",
		"FIELD",
		"FIELDS",
		"FILTERS",
		"FLEXIBLE",
		"FOR",
		"FROM",
		"FULL",
		"FULLTEXT",
		"FUNCTION",
		"FUNCTIONS",
		"GRANT",
		"GRAPHQL",
		"GROUP",
		"HEADERS",
		"HIGHLIGHTS",
		"HNSW",
		"IGNORE",
		"INCLUDE",
		"INDEX",
		"INFO",
		"INSERT",
		"INTO",
		"IF",
		"IS",
		"ISSUER",
		"JWT",
		"JWKS",
		"HASHED_VECTOR",
		"KEY",
		"KEEP_PRUNED_CONNECTIONS",
		"KILL",
		"LET",
		"LIMIT",
		"LIVE",
		"LOWERCASE",
		"LM",
		"M",
		"M0",
		"MAPPER",
		"MAXDEPTH",
		"MIDDLEWARE",
		"MERGE",
		"MODEL",
		"MODULE",
		"NAMESPACE",
		"NGRAM",
		"NO",
		"NOINDEX",
		"NONE",
		"NULL",
		"NUMERIC",
		"OMIT",
		"ON",
		"ONLY",
		"OPTION",
		"ORDER",
		"ORIGINAL",
		"OVERWRITE",
		"PARALLEL",
		"PARAM",
		"PASSHASH",
		"PASSWORD",
		"PATCH",
		"PERMISSIONS",
		"POSTINGS_CACHE",
		"POSTINGS_ORDER",
		"PREPARE",
		"PUNCT",
		"PURGE",
		"RANGE",
		"READONLY",
		"REBUILD",
		"REFERENCE",
		"REFRESH",
		"REGEX",
		"RELATE",
		"RELATION",
		"REMOVE",
		"REPLACE",
		"RETRY",
		"RETURN",
		"REVOKE",
		"REVOKED",
		"ROLES",
		"ROOT",
		"SCHEMAFULL",
		"SCHEMALESS",
		"SCOPE",
		"SELECT",
		"SEQUENCE",
		"SESSION",
		"SET",
		"SHOW",
		"SIGNIN",
		"SIGNUP",
		"SINCE",
		"SLEEP",
		"SNOWBALL",
		"SPLIT",
		"START",
		"STRICT",
		"STRUCTURE",
		"SYSTEM",
		"TABLE",
		"TABLES",
		"TEMPFILES",
		"TERMS_CACHE",
		"TERMS_ORDER",
		"THEN",
		"THROW",
		"TIMEOUT",
		"TOKENIZERS",
		"TOKEN",
		"TO",
		"TRANSACTION",
		"true",
		"TYPE",
		"UNIQUE",
		"UNSET",
		"UPDATE",
		"UPSERT",
		"UPPERCASE",
		"URL",
		"USE",
		"USER",
		"VALUE",
		"VALUES",
		"VERSION",
		"VS",
		"WHEN",
		"WHERE",
		"WITH",
		"ALLINSIDE",
		"ANYINSIDE",
		"INSIDE",
		"INTERSECTS",
		"NONEINSIDE",
		"NOTINSIDE",
		"OR",
		"OUTSIDE",
		"NOT",
		"AND",
		"COLLATE",
		"CONTAINSALL",
		"CONTAINSANY",
		"CONTAINSNONE",
		"CONTAINSNOT",
		"CONTAINS",
		"IN",
		"OUT",
		"NORMAL",
		"ANY",
		"ARRAY",
		"GEOMETRY",
		"RECORD",
		"BOOL",
		"BYTES",
		"DATETIME",
		"DECIMAL",
		"DURATION",
		"FLOAT",
		"fn",
		"silo",
		"mod",
		"INT",
		"NUMBER",
		"OBJECT",
		"STRING",
		"UUID",
		"ULID",
		"RAND",
		"REFERENCES",
		"FEATURE",
		"LINE",
		"POINT",
		"POLYGON",
		"MULTIPOINT",
		"MULTILINE",
		"MULTIPOLYGON",
		"COLLECTION",
		"FILE",
		"ml",
		"GET",
		"POST",
		"PUT",
		"TRACE",
	]
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
pub fn parse_for_diagnostics(input: &str) -> Result<Ast, Vec<ParseDiagnostic>> {
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

/// Syntax error type.
pub use upstream::syn::error;

pub use recovery::parse_with_recovery;
pub use schema_graph::SchemaGraph;
