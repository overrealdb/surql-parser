//! Schema graph — semantic model built from SurrealQL definitions.
//!
//! Provides a queryable graph of tables, fields, indexes, functions, and their
//! relationships. Built from parsed `.surql` files.
//!
//! # Example
//!
//! ```
//! use surql_parser::SchemaGraph;
//!
//! let schema = SchemaGraph::from_source("
//!     DEFINE TABLE user SCHEMAFULL;
//!     DEFINE FIELD name ON user TYPE string;
//!     DEFINE FIELD age ON user TYPE int;
//!     DEFINE INDEX email_idx ON user FIELDS email UNIQUE;
//!     DEFINE FUNCTION fn::greet($name: string) -> string { RETURN 'Hello, ' + $name; };
//! ").unwrap();
//!
//! assert_eq!(schema.table_names().count(), 1);
//! assert_eq!(schema.fields_of("user").len(), 2);
//! assert_eq!(schema.indexes_of("user").len(), 1);
//! assert!(schema.function("greet").is_some());
//! ```

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use surrealdb_types::{SqlFormat, ToSql};

/// Source location of a definition in a `.surql` file.
#[derive(Debug, Clone)]
pub struct SourceLocation {
	pub file: Arc<Path>,
	pub offset: usize,
	pub len: usize,
}

/// A parsed table definition.
#[derive(Debug, Clone)]
pub struct TableDef {
	pub name: String,
	pub full: bool,
	pub table_type: String,
	pub comment: Option<String>,
	pub fields: Vec<FieldDef>,
	pub indexes: Vec<IndexDef>,
	pub events: Vec<EventDef>,
	pub source: Option<SourceLocation>,
	pub ns: Option<String>,
	pub db: Option<String>,
}

/// A parsed field definition.
#[derive(Debug, Clone)]
pub struct FieldDef {
	pub name: String,
	pub kind: Option<String>,
	pub record_links: Vec<String>,
	pub default: Option<String>,
	pub readonly: bool,
	pub comment: Option<String>,
	pub source: Option<SourceLocation>,
}

/// A parsed index definition.
#[derive(Debug, Clone)]
pub struct IndexDef {
	pub name: String,
	pub columns: Vec<String>,
	pub unique: bool,
	pub comment: Option<String>,
	pub source: Option<SourceLocation>,
}

/// A parsed event definition.
#[derive(Debug, Clone)]
pub struct EventDef {
	pub name: String,
	pub comment: Option<String>,
	pub source: Option<SourceLocation>,
}

/// A parsed function definition.
#[derive(Debug, Clone)]
pub struct FunctionDef {
	pub name: String,
	pub args: Vec<(String, String)>,
	pub returns: Option<String>,
	pub comment: Option<String>,
	pub source: Option<SourceLocation>,
}

/// A parsed param definition.
#[derive(Debug, Clone)]
pub struct ParamDef {
	pub name: String,
	pub source: Option<SourceLocation>,
}

/// A semantic graph of SurrealQL schema definitions.
///
/// Built from `.surql` files, provides fast lookups for tables, fields,
/// functions, and their relationships.
#[derive(Debug, Clone, Default)]
pub struct SchemaGraph {
	tables: HashMap<String, TableDef>,
	functions: HashMap<String, FunctionDef>,
	params: HashMap<String, ParamDef>,
	/// field_name -> [(table_name, field_idx)] for O(1) field lookups.
	field_index: HashMap<String, Vec<(String, usize)>>,
}

impl SchemaGraph {
	/// Build a schema graph from a SurrealQL string.
	pub fn from_source(input: &str) -> anyhow::Result<Self> {
		let defs = crate::extract_definitions(input)?;
		Ok(Self::from_definitions(&defs))
	}

	/// Build a schema graph by walking all `.surql` files in a directory.
	///
	/// Source locations are tracked per file for go-to-definition support.
	pub fn from_files(dir: &Path) -> anyhow::Result<Self> {
		let mut graph = Self::default();
		Self::collect_files(dir, &mut graph)?;
		Ok(graph)
	}

	fn collect_files(dir: &Path, graph: &mut Self) -> anyhow::Result<()> {
		let mut entries: Vec<_> = std::fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
		entries.sort_by_key(|e| e.file_name());

		for entry in entries {
			let path = entry.path();
			// Skip test fixtures, build artifacts, and node_modules
			if path.is_dir() {
				let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
				if matches!(
					name,
					"target" | "node_modules" | "fixtures" | ".git" | "build"
				) {
					continue;
				}
			}
			if path.is_file() && path.extension().is_some_and(|ext| ext == "surql") {
				let content = match std::fs::read_to_string(&path) {
					Ok(c) => c,
					Err(e) => {
						tracing::warn!("Skipping {}: {e}", path.display());
						continue;
					}
				};
				// Use recovery parser so a single broken file doesn't abort the scan
				let (stmts, _) = crate::parse_with_recovery(&content);
				match crate::extract_definitions_from_ast(&stmts) {
					Ok(defs) => {
						let mut file_graph = Self::from_definitions(&defs);
						file_graph.attach_source_locations(&content, &path);
						graph.merge(file_graph);
					}
					Err(e) => {
						tracing::warn!("Skipping defs from {}: {e}", path.display());
					}
				}
			} else if path.is_dir() {
				Self::collect_files(&path, graph)?;
			}
		}
		Ok(())
	}

	/// Scan source text via the lexer for DEFINE statement positions.
	pub fn attach_source_locations(&mut self, source: &str, file: &Path) {
		use crate::upstream::syn::lexer::Lexer;
		use crate::upstream::syn::token::TokenKind;

		let bytes = source.as_bytes();
		if bytes.len() > u32::MAX as usize {
			return;
		}

		let tokens: Vec<_> = Lexer::new(bytes).collect();
		let file: Arc<Path> = Arc::from(file);

		for i in 0..tokens.len() {
			let define_token = &tokens[i];
			if token_text(source, define_token).to_uppercase() != "DEFINE" {
				continue;
			}

			// Skip OVERWRITE/IF NOT EXISTS
			let mut j = i + 1;
			while j < tokens.len() {
				let t = token_text(source, &tokens[j]).to_uppercase();
				if t == "OVERWRITE" || t == "IF" || t == "NOT" || t == "EXISTS" {
					j += 1;
				} else {
					break;
				}
			}
			if j >= tokens.len() {
				continue;
			}

			let kind_text = token_text(source, &tokens[j]).to_uppercase();

			if kind_text == "TABLE" && j + 1 < tokens.len() {
				let name_token = &tokens[j + 1];
				let name = token_text(source, name_token);
				if let Some(table) = self.tables.get_mut(name).filter(|t| t.source.is_none()) {
					table.source = Some(SourceLocation {
						file: Arc::clone(&file),
						offset: define_token.span.offset as usize,
						len: (name_token.span.offset + name_token.span.len
							- define_token.span.offset) as usize,
					});
				}
			}

			if kind_text == "FUNCTION" && j + 1 < tokens.len() {
				let fn_start = j + 1;
				let mut fn_end = fn_start;
				while fn_end < tokens.len() {
					let tk = tokens[fn_end].kind;
					if tk == TokenKind::Identifier
						|| tk == TokenKind::PathSeperator
						|| matches!(tk, TokenKind::Keyword(_))
					{
						fn_end += 1;
					} else {
						break;
					}
				}
				if fn_end > fn_start {
					let name_start = tokens[fn_start].span.offset as usize;
					let name_end =
						(tokens[fn_end - 1].span.offset + tokens[fn_end - 1].span.len) as usize;
					let full_name = &source[name_start..name_end];
					let fn_name = full_name.strip_prefix("fn::").unwrap_or(full_name);
					if let Some(func) = self
						.functions
						.get_mut(fn_name)
						.filter(|f| f.source.is_none())
					{
						func.source = Some(SourceLocation {
							file: Arc::clone(&file),
							offset: define_token.span.offset as usize,
							len: name_end - define_token.span.offset as usize,
						});
					}
				}
			}

			if kind_text == "FIELD" && j + 1 < tokens.len() {
				let field_name = token_text(source, &tokens[j + 1]);
				for k in (j + 2)..tokens.len().min(j + 6) {
					if token_text(source, &tokens[k]).to_uppercase() == "ON" && k + 1 < tokens.len()
					{
						let table_name = token_text(source, &tokens[k + 1]);
						if let Some(table) = self.tables.get_mut(table_name) {
							for field in &mut table.fields {
								if field.name == field_name && field.source.is_none() {
									field.source = Some(SourceLocation {
										file: Arc::clone(&file),
										offset: define_token.span.offset as usize,
										len: (tokens[(k + 1).min(tokens.len() - 1)].span.offset
											+ tokens[(k + 1).min(tokens.len() - 1)].span.len
											- define_token.span.offset) as usize,
									});
								}
							}
						}
						break;
					}
				}
			}
		}
	}

	/// Merge another schema graph into this one.
	///
	/// Fields, indexes, and events are deduplicated by name.
	/// SCHEMAFULL wins over SCHEMALESS (once a table is marked SCHEMAFULL, it stays).
	pub fn merge(&mut self, other: SchemaGraph) {
		for (name, table) in other.tables {
			if let Some(existing) = self.tables.get_mut(&name) {
				if table.full {
					existing.full = true;
				}
				for field in table.fields {
					if !existing.fields.iter().any(|f| f.name == field.name) {
						let field_idx = existing.fields.len();
						self.field_index
							.entry(field.name.clone())
							.or_default()
							.push((name.clone(), field_idx));
						existing.fields.push(field);
					}
				}
				for index in table.indexes {
					if !existing.indexes.iter().any(|i| i.name == index.name) {
						existing.indexes.push(index);
					}
				}
				for event in table.events {
					if !existing.events.iter().any(|e| e.name == event.name) {
						existing.events.push(event);
					}
				}
				if table.source.is_some() {
					existing.source = table.source;
				}
			} else {
				for (field_idx, field) in table.fields.iter().enumerate() {
					self.field_index
						.entry(field.name.clone())
						.or_default()
						.push((name.clone(), field_idx));
				}
				self.tables.insert(name, table);
			}
		}
		self.functions.extend(other.functions);
		self.params.extend(other.params);
	}

	pub fn from_definitions(defs: &crate::SchemaDefinitions) -> Self {
		let mut graph = Self::default();

		for t in &defs.tables {
			let name = expr_to_string(&t.name);
			graph.tables.insert(
				name.clone(),
				TableDef {
					name,
					full: t.full,
					table_type: format!("{:?}", t.table_type),
					comment: extract_comment(&t.comment),
					fields: Vec::new(),
					indexes: Vec::new(),
					events: Vec::new(),
					source: None,
					ns: defs.current_ns.clone(),
					db: defs.current_db.clone(),
				},
			);
		}

		for f in &defs.fields {
			let field_name = expr_to_string(&f.name);
			let table_name = expr_to_string(&f.what);
			let kind_str = f.field_kind.as_ref().map(|k| {
				let mut s = String::new();
				k.fmt_sql(&mut s, SqlFormat::SingleLine);
				s
			});
			let record_links = f
				.field_kind
				.as_ref()
				.map(extract_record_links)
				.unwrap_or_default();
			let default_str = match &f.default {
				crate::upstream::sql::statements::define::DefineDefault::None => None,
				crate::upstream::sql::statements::define::DefineDefault::Always(e)
				| crate::upstream::sql::statements::define::DefineDefault::Set(e) => Some(expr_to_string(e)),
			};

			let def = FieldDef {
				name: field_name,
				kind: kind_str,
				record_links,
				default: default_str,
				readonly: f.readonly,
				comment: extract_comment(&f.comment),
				source: None,
			};

			if let Some(table) = graph.tables.get_mut(&table_name) {
				table.fields.push(def);
			} else {
				graph.tables.insert(
					table_name.clone(),
					TableDef {
						name: table_name,
						full: false,
						table_type: "Any".into(),
						comment: None,
						fields: vec![def],
						indexes: Vec::new(),
						events: Vec::new(),
						source: None,
						ns: defs.current_ns.clone(),
						db: defs.current_db.clone(),
					},
				);
			}
		}

		for idx in &defs.indexes {
			let index_name = expr_to_string(&idx.name);
			let table_name = expr_to_string(&idx.what);
			let columns: Vec<String> = idx.cols.iter().map(expr_to_string).collect();
			let unique = matches!(idx.index, crate::upstream::sql::index::Index::Uniq);

			let def = IndexDef {
				name: index_name,
				columns,
				unique,
				comment: extract_comment(&idx.comment),
				source: None,
			};

			if let Some(table) = graph.tables.get_mut(&table_name) {
				table.indexes.push(def);
			}
		}

		for ev in &defs.events {
			let event_name = expr_to_string(&ev.name);
			let table_name = expr_to_string(&ev.target_table);

			let def = EventDef {
				name: event_name,
				comment: extract_comment(&ev.comment),
				source: None,
			};

			if let Some(table) = graph.tables.get_mut(&table_name) {
				table.events.push(def);
			}
		}

		for func in &defs.functions {
			let args: Vec<(String, String)> = func
				.args
				.iter()
				.map(|(name, kind)| {
					let mut kind_str = String::new();
					kind.fmt_sql(&mut kind_str, SqlFormat::SingleLine);
					(name.clone(), kind_str)
				})
				.collect();

			let returns = func.returns.as_ref().map(|k| {
				let mut s = String::new();
				k.fmt_sql(&mut s, SqlFormat::SingleLine);
				s
			});

			graph.functions.insert(
				func.name.clone(),
				FunctionDef {
					name: func.name.clone(),
					args,
					returns,
					comment: extract_comment(&func.comment),
					source: None,
				},
			);
		}

		for p in &defs.params {
			graph.params.insert(
				p.name.clone(),
				ParamDef {
					name: p.name.clone(),
					source: None,
				},
			);
		}

		graph.rebuild_field_index();
		graph
	}

	fn rebuild_field_index(&mut self) {
		self.field_index.clear();
		for (table_name, table) in &self.tables {
			for (field_idx, field) in table.fields.iter().enumerate() {
				self.field_index
					.entry(field.name.clone())
					.or_default()
					.push((table_name.clone(), field_idx));
			}
		}
	}

	// ─── Lookups ───

	/// Iterate over all table names in the schema.
	pub fn table_names(&self) -> impl Iterator<Item = &str> {
		self.tables.keys().map(|s| s.as_str())
	}

	/// Get table definition by name.
	pub fn table(&self, name: &str) -> Option<&TableDef> {
		self.tables.get(name)
	}

	/// Fields defined on a table.
	pub fn fields_of(&self, table: &str) -> &[FieldDef] {
		self.tables
			.get(table)
			.map(|t| t.fields.as_slice())
			.unwrap_or(&[])
	}

	/// Indexes defined on a table.
	pub fn indexes_of(&self, table: &str) -> &[IndexDef] {
		self.tables
			.get(table)
			.map(|t| t.indexes.as_slice())
			.unwrap_or(&[])
	}

	/// Events defined on a table.
	pub fn events_of(&self, table: &str) -> &[EventDef] {
		self.tables
			.get(table)
			.map(|t| t.events.as_slice())
			.unwrap_or(&[])
	}

	/// Get function definition by name (without the `fn::` prefix).
	pub fn function(&self, name: &str) -> Option<&FunctionDef> {
		self.functions.get(name)
	}

	/// Iterate over all function names (without `fn::` prefix).
	pub fn function_names(&self) -> impl Iterator<Item = &str> {
		self.functions.keys().map(|s| s.as_str())
	}

	/// Iterate over all defined param names.
	pub fn param_names(&self) -> impl Iterator<Item = &str> {
		self.params.keys().map(|s| s.as_str())
	}

	/// Find a field by name across all tables. Returns (table_name, field_def).
	///
	/// Uses a pre-built HashMap index for O(1) lookup by field name,
	/// instead of scanning all tables.
	pub fn find_field(&self, field_name: &str) -> Vec<(&str, &FieldDef)> {
		let Some(entries) = self.field_index.get(field_name) else {
			return Vec::new();
		};
		entries
			.iter()
			.filter_map(|(table_name, field_idx)| {
				let table = self.tables.get(table_name)?;
				let field = table.fields.get(*field_idx)?;
				Some((table_name.as_str(), field))
			})
			.collect()
	}

	/// Find a field on a specific table.
	pub fn field_on(&self, table: &str, field_name: &str) -> Option<&FieldDef> {
		self.tables
			.get(table)
			.and_then(|t| t.fields.iter().find(|f| f.name == field_name))
	}
}

// ─── Helpers ───

fn token_text<'a>(source: &'a str, token: &crate::upstream::syn::token::Token) -> &'a str {
	let start = token.span.offset as usize;
	let end = (token.span.offset + token.span.len) as usize;
	if end <= source.len() {
		&source[start..end]
	} else {
		""
	}
}

fn expr_to_string(expr: &crate::Expr) -> String {
	let mut s = String::new();
	expr.fmt_sql(&mut s, SqlFormat::SingleLine);
	s.trim_matches('`')
		.trim_matches('⟨')
		.trim_matches('⟩')
		.to_string()
}

fn extract_comment(expr: &crate::Expr) -> Option<String> {
	use crate::upstream::sql::Literal;
	match expr {
		crate::Expr::Literal(Literal::None) => None,
		crate::Expr::Literal(Literal::String(s)) => Some(s.clone()),
		other => {
			let s = expr_to_string(other);
			let trimmed = s.trim_matches('\'').trim_matches('"');
			if trimmed.is_empty() {
				None
			} else {
				Some(trimmed.to_string())
			}
		}
	}
}

fn extract_record_links(kind: &crate::Kind) -> Vec<String> {
	let mut s = String::new();
	kind.fmt_sql(&mut s, SqlFormat::SingleLine);

	let mut links = Vec::new();
	let mut remaining = s.as_str();
	while let Some(pos) = remaining.find("record<") {
		let after = &remaining[pos + 7..];
		if let Some(end) = after.find('>') {
			let tables_str = &after[..end];
			for table in tables_str.split('|') {
				let table = table.trim();
				if !table.is_empty() {
					links.push(table.to_string());
				}
			}
		}
		remaining = &remaining[pos + 7..];
	}
	links
}
