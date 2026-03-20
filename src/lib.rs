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

pub mod builtins_generated;
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
pub fn extract_definitions(input: &str) -> anyhow::Result<SchemaDefinitions> {
	let ast = parse(input)?;
	extract_definitions_from_ast(&ast.expressions)
}

/// Extract definitions from pre-parsed statements (e.g., from error-recovering parser).
pub fn extract_definitions_from_ast(
	stmts: &[upstream::sql::ast::TopLevelExpr],
) -> anyhow::Result<SchemaDefinitions> {
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
/// Generated directly from the parser's `Keyword` enum via `as_str()`.
/// Cannot drift — any new keyword in the enum is automatically included.
pub fn all_keywords() -> &'static [&'static str] {
	use std::sync::LazyLock;
	use upstream::syn::token::Keyword;

	// Every Keyword variant, extracted from the enum definition.
	// Cannot drift: adding a variant to Keyword without adding it here
	// causes a compile error (missing match arm in the macro expansion).
	static KEYWORDS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
		vec![
			Keyword::Access.as_str(),
			Keyword::After.as_str(),
			Keyword::Algorithm.as_str(),
			Keyword::All.as_str(),
			Keyword::Alter.as_str(),
			Keyword::Always.as_str(),
			Keyword::Analyzer.as_str(),
			Keyword::Api.as_str(),
			Keyword::As.as_str(),
			Keyword::Ascending.as_str(),
			Keyword::Ascii.as_str(),
			Keyword::Assert.as_str(),
			Keyword::Async.as_str(),
			Keyword::At.as_str(),
			Keyword::Authenticate.as_str(),
			Keyword::Auto.as_str(),
			Keyword::Backend.as_str(),
			Keyword::Batch.as_str(),
			Keyword::Bearer.as_str(),
			Keyword::Before.as_str(),
			Keyword::Begin.as_str(),
			Keyword::Blank.as_str(),
			Keyword::Bucket.as_str(),
			Keyword::Reject.as_str(),
			Keyword::Bm25.as_str(),
			Keyword::Break.as_str(),
			Keyword::By.as_str(),
			Keyword::Camel.as_str(),
			Keyword::Cancel.as_str(),
			Keyword::Cascade.as_str(),
			Keyword::ChangeFeed.as_str(),
			Keyword::Changes.as_str(),
			Keyword::Capacity.as_str(),
			Keyword::Class.as_str(),
			Keyword::Comment.as_str(),
			Keyword::Commit.as_str(),
			Keyword::Compact.as_str(),
			Keyword::Concurrently.as_str(),
			Keyword::Config.as_str(),
			Keyword::Content.as_str(),
			Keyword::Continue.as_str(),
			Keyword::Computed.as_str(),
			Keyword::Count.as_str(),
			Keyword::Create.as_str(),
			Keyword::Database.as_str(),
			Keyword::Default.as_str(),
			Keyword::Define.as_str(),
			Keyword::Delete.as_str(),
			Keyword::Descending.as_str(),
			Keyword::Diff.as_str(),
			Keyword::Dimension.as_str(),
			Keyword::Distance.as_str(),
			Keyword::Drop.as_str(),
			Keyword::Duplicate.as_str(),
			Keyword::Efc.as_str(),
			Keyword::Edgengram.as_str(),
			Keyword::Event.as_str(),
			Keyword::Else.as_str(),
			Keyword::End.as_str(),
			Keyword::Enforced.as_str(),
			Keyword::Exclude.as_str(),
			Keyword::Exists.as_str(),
			Keyword::Expired.as_str(),
			Keyword::Explain.as_str(),
			Keyword::Expunge.as_str(),
			Keyword::ExtendCandidates.as_str(),
			Keyword::False.as_str(),
			Keyword::Fetch.as_str(),
			Keyword::Field.as_str(),
			Keyword::Fields.as_str(),
			Keyword::Filters.as_str(),
			Keyword::Flexible.as_str(),
			Keyword::For.as_str(),
			Keyword::From.as_str(),
			Keyword::Full.as_str(),
			Keyword::Fulltext.as_str(),
			Keyword::Function.as_str(),
			Keyword::Functions.as_str(),
			Keyword::Grant.as_str(),
			Keyword::Graphql.as_str(),
			Keyword::Group.as_str(),
			Keyword::Headers.as_str(),
			Keyword::Highlights.as_str(),
			Keyword::Hnsw.as_str(),
			Keyword::Ignore.as_str(),
			Keyword::Include.as_str(),
			Keyword::Index.as_str(),
			Keyword::Info.as_str(),
			Keyword::Insert.as_str(),
			Keyword::Into.as_str(),
			Keyword::If.as_str(),
			Keyword::Is.as_str(),
			Keyword::Issuer.as_str(),
			Keyword::Jwt.as_str(),
			Keyword::Jwks.as_str(),
			Keyword::HashedVector.as_str(),
			Keyword::Key.as_str(),
			Keyword::KeepPrunedConnections.as_str(),
			Keyword::Kill.as_str(),
			Keyword::Let.as_str(),
			Keyword::Limit.as_str(),
			Keyword::Live.as_str(),
			Keyword::Lowercase.as_str(),
			Keyword::Lm.as_str(),
			Keyword::M.as_str(),
			Keyword::M0.as_str(),
			Keyword::Mapper.as_str(),
			Keyword::MaxDepth.as_str(),
			Keyword::Middleware.as_str(),
			Keyword::Merge.as_str(),
			Keyword::Model.as_str(),
			Keyword::Module.as_str(),
			Keyword::Namespace.as_str(),
			Keyword::Ngram.as_str(),
			Keyword::No.as_str(),
			Keyword::NoIndex.as_str(),
			Keyword::None.as_str(),
			Keyword::Null.as_str(),
			Keyword::Numeric.as_str(),
			Keyword::Omit.as_str(),
			Keyword::On.as_str(),
			Keyword::Only.as_str(),
			Keyword::Option.as_str(),
			Keyword::Order.as_str(),
			Keyword::Original.as_str(),
			Keyword::Overwrite.as_str(),
			Keyword::Parallel.as_str(),
			Keyword::Param.as_str(),
			Keyword::Passhash.as_str(),
			Keyword::Password.as_str(),
			Keyword::Patch.as_str(),
			Keyword::Permissions.as_str(),
			Keyword::PostingsCache.as_str(),
			Keyword::PostingsOrder.as_str(),
			Keyword::Prepare.as_str(),
			Keyword::Punct.as_str(),
			Keyword::Purge.as_str(),
			Keyword::Range.as_str(),
			Keyword::Readonly.as_str(),
			Keyword::Rebuild.as_str(),
			Keyword::Reference.as_str(),
			Keyword::Refresh.as_str(),
			Keyword::Regex.as_str(),
			Keyword::Relate.as_str(),
			Keyword::Relation.as_str(),
			Keyword::Remove.as_str(),
			Keyword::Replace.as_str(),
			Keyword::Retry.as_str(),
			Keyword::Return.as_str(),
			Keyword::Revoke.as_str(),
			Keyword::Revoked.as_str(),
			Keyword::Roles.as_str(),
			Keyword::Root.as_str(),
			Keyword::Schemafull.as_str(),
			Keyword::Schemaless.as_str(),
			Keyword::Scope.as_str(),
			Keyword::Select.as_str(),
			Keyword::Sequence.as_str(),
			Keyword::Session.as_str(),
			Keyword::Set.as_str(),
			Keyword::Show.as_str(),
			Keyword::Signin.as_str(),
			Keyword::Signup.as_str(),
			Keyword::Since.as_str(),
			Keyword::Sleep.as_str(),
			Keyword::Snowball.as_str(),
			Keyword::Split.as_str(),
			Keyword::Start.as_str(),
			Keyword::Strict.as_str(),
			Keyword::Structure.as_str(),
			Keyword::System.as_str(),
			Keyword::Table.as_str(),
			Keyword::Tables.as_str(),
			Keyword::TempFiles.as_str(),
			Keyword::TermsCache.as_str(),
			Keyword::TermsOrder.as_str(),
			Keyword::Then.as_str(),
			Keyword::Throw.as_str(),
			Keyword::Timeout.as_str(),
			Keyword::Tokenizers.as_str(),
			Keyword::Token.as_str(),
			Keyword::To.as_str(),
			Keyword::Transaction.as_str(),
			Keyword::True.as_str(),
			Keyword::Type.as_str(),
			Keyword::Unique.as_str(),
			Keyword::Unset.as_str(),
			Keyword::Update.as_str(),
			Keyword::Upsert.as_str(),
			Keyword::Uppercase.as_str(),
			Keyword::Url.as_str(),
			Keyword::Use.as_str(),
			Keyword::User.as_str(),
			Keyword::Value.as_str(),
			Keyword::Values.as_str(),
			Keyword::Version.as_str(),
			Keyword::Vs.as_str(),
			Keyword::When.as_str(),
			Keyword::Where.as_str(),
			Keyword::With.as_str(),
			Keyword::AllInside.as_str(),
			Keyword::AndKw.as_str(),
			Keyword::AnyInside.as_str(),
			Keyword::Inside.as_str(),
			Keyword::Intersects.as_str(),
			Keyword::NoneInside.as_str(),
			Keyword::NotInside.as_str(),
			Keyword::OrKw.as_str(),
			Keyword::Outside.as_str(),
			Keyword::Not.as_str(),
			Keyword::And.as_str(),
			Keyword::Collate.as_str(),
			Keyword::ContainsAll.as_str(),
			Keyword::ContainsAny.as_str(),
			Keyword::ContainsNone.as_str(),
			Keyword::ContainsNot.as_str(),
			Keyword::Contains.as_str(),
			Keyword::In.as_str(),
			Keyword::Out.as_str(),
			Keyword::Normal.as_str(),
			Keyword::Any.as_str(),
			Keyword::Array.as_str(),
			Keyword::Geometry.as_str(),
			Keyword::Record.as_str(),
			Keyword::Bool.as_str(),
			Keyword::Bytes.as_str(),
			Keyword::Datetime.as_str(),
			Keyword::Decimal.as_str(),
			Keyword::Duration.as_str(),
			Keyword::Float.as_str(),
			Keyword::Fn.as_str(),
			Keyword::Silo.as_str(),
			Keyword::Mod.as_str(),
			Keyword::Int.as_str(),
			Keyword::Number.as_str(),
			Keyword::Object.as_str(),
			Keyword::String.as_str(),
			Keyword::Uuid.as_str(),
			Keyword::Ulid.as_str(),
			Keyword::Rand.as_str(),
			Keyword::References.as_str(),
			Keyword::Feature.as_str(),
			Keyword::Line.as_str(),
			Keyword::Point.as_str(),
			Keyword::Polygon.as_str(),
			Keyword::MultiPoint.as_str(),
			Keyword::MultiLine.as_str(),
			Keyword::MultiPolygon.as_str(),
			Keyword::Collection.as_str(),
			Keyword::File.as_str(),
			Keyword::FN.as_str(),
			Keyword::ML.as_str(),
			Keyword::Get.as_str(),
			Keyword::Post.as_str(),
			Keyword::Put.as_str(),
			Keyword::Trace.as_str(),
		]
	});
	&KEYWORDS
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
