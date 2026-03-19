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
//! assert_eq!(schema.table_names().len(), 1);
//! assert_eq!(schema.fields_of("user").len(), 2);
//! assert_eq!(schema.indexes_of("user").len(), 1);
//! assert!(schema.function("greet").is_some());
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use surrealdb_types::{SqlFormat, ToSql};

/// Source location of a definition in a `.surql` file.
#[derive(Debug, Clone)]
pub struct SourceLocation {
	pub file: PathBuf,
	pub offset: usize,
	pub len: usize,
}

/// Information about a table definition.
#[derive(Debug, Clone)]
pub struct TableInfo {
	pub name: String,
	pub full: bool,
	pub table_type: String,
	pub fields: Vec<FieldInfo>,
	pub indexes: Vec<IndexInfo>,
	pub events: Vec<EventInfo>,
	pub source: Option<SourceLocation>,
}

/// Information about a field definition.
#[derive(Debug, Clone)]
pub struct FieldInfo {
	pub name: String,
	pub table: String,
	pub kind: Option<String>,
	pub record_links: Vec<String>,
	pub default: Option<String>,
	pub readonly: bool,
	pub source: Option<SourceLocation>,
}

/// Information about an index definition.
#[derive(Debug, Clone)]
pub struct IndexInfo {
	pub name: String,
	pub table: String,
	pub columns: Vec<String>,
	pub unique: bool,
	pub source: Option<SourceLocation>,
}

/// Information about an event definition.
#[derive(Debug, Clone)]
pub struct EventInfo {
	pub name: String,
	pub table: String,
	pub source: Option<SourceLocation>,
}

/// Information about a function definition.
#[derive(Debug, Clone)]
pub struct FunctionInfo {
	pub name: String,
	pub args: Vec<(String, String)>,
	pub returns: Option<String>,
	pub source: Option<SourceLocation>,
}

/// Information about a param definition.
#[derive(Debug, Clone)]
pub struct ParamInfo {
	pub name: String,
	pub source: Option<SourceLocation>,
}

/// A semantic graph of SurrealQL schema definitions.
///
/// Built from `.surql` files, provides fast lookups for tables, fields,
/// functions, and their relationships.
#[derive(Debug, Clone, Default)]
pub struct SchemaGraph {
	tables: HashMap<String, TableInfo>,
	functions: HashMap<String, FunctionInfo>,
	params: HashMap<String, ParamInfo>,
}

impl SchemaGraph {
	/// Build a schema graph from a SurrealQL string.
	pub fn from_source(input: &str) -> anyhow::Result<Self> {
		let defs = crate::extract_definitions(input)?;
		Ok(Self::from_definitions(&defs))
	}

	/// Build a schema graph by walking all `.surql` files in a directory.
	pub fn from_files(dir: &Path) -> anyhow::Result<Self> {
		let mut all_sql = String::new();
		for entry in std::fs::read_dir(dir)? {
			let entry = entry?;
			let path = entry.path();
			if path.is_file() && path.extension().is_some_and(|ext| ext == "surql") {
				let content = std::fs::read_to_string(&path)?;
				all_sql.push_str(&content);
				all_sql.push('\n');
			} else if path.is_dir() {
				// Recurse into subdirectories
				let sub_graph = Self::from_files(&path)?;
				let mut graph = Self::from_source(&all_sql)?;
				graph.merge(sub_graph);
				return Ok(graph);
			}
		}
		Self::from_source(&all_sql)
	}

	/// Merge another schema graph into this one (last write wins).
	pub fn merge(&mut self, other: SchemaGraph) {
		for (name, table) in other.tables {
			if let Some(existing) = self.tables.get_mut(&name) {
				existing.fields.extend(table.fields);
				existing.indexes.extend(table.indexes);
				existing.events.extend(table.events);
				if table.source.is_some() {
					existing.source = table.source;
				}
			} else {
				self.tables.insert(name, table);
			}
		}
		self.functions.extend(other.functions);
		self.params.extend(other.params);
	}

	fn from_definitions(defs: &crate::SchemaDefinitions) -> Self {
		let mut graph = Self::default();

		// Tables
		for t in &defs.tables {
			let name = expr_to_string(&t.name);
			let table_type = format!("{:?}", t.table_type);
			graph.tables.insert(
				name.clone(),
				TableInfo {
					name,
					full: t.full,
					table_type,
					fields: Vec::new(),
					indexes: Vec::new(),
					events: Vec::new(),
					source: None,
				},
			);
		}

		// Fields
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

			let info = FieldInfo {
				name: field_name,
				table: table_name.clone(),
				kind: kind_str,
				record_links,
				default: default_str,
				readonly: f.readonly,
				source: None,
			};

			if let Some(table) = graph.tables.get_mut(&table_name) {
				table.fields.push(info);
			} else {
				// Table not explicitly defined, create implicit entry
				graph.tables.insert(
					table_name.clone(),
					TableInfo {
						name: table_name,
						full: false,
						table_type: "Any".into(),
						fields: vec![info],
						indexes: Vec::new(),
						events: Vec::new(),
						source: None,
					},
				);
			}
		}

		// Indexes
		for idx in &defs.indexes {
			let index_name = expr_to_string(&idx.name);
			let table_name = expr_to_string(&idx.what);
			let columns: Vec<String> = idx.cols.iter().map(expr_to_string).collect();
			let unique = matches!(idx.index, crate::upstream::sql::index::Index::Uniq);

			let info = IndexInfo {
				name: index_name,
				table: table_name.clone(),
				columns,
				unique,
				source: None,
			};

			if let Some(table) = graph.tables.get_mut(&table_name) {
				table.indexes.push(info);
			}
		}

		// Events
		for ev in &defs.events {
			let event_name = expr_to_string(&ev.name);
			let table_name = expr_to_string(&ev.target_table);

			let info = EventInfo {
				name: event_name,
				table: table_name.clone(),
				source: None,
			};

			if let Some(table) = graph.tables.get_mut(&table_name) {
				table.events.push(info);
			}
		}

		// Functions
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
				FunctionInfo {
					name: func.name.clone(),
					args,
					returns,
					source: None,
				},
			);
		}

		// Params
		for p in &defs.params {
			graph.params.insert(
				p.name.clone(),
				ParamInfo {
					name: p.name.clone(),
					source: None,
				},
			);
		}

		graph
	}

	// ─── Lookups ───

	/// All table names in the schema.
	pub fn table_names(&self) -> Vec<&str> {
		self.tables.keys().map(|s| s.as_str()).collect()
	}

	/// Get table info by name.
	pub fn table(&self, name: &str) -> Option<&TableInfo> {
		self.tables.get(name)
	}

	/// Fields defined on a table.
	pub fn fields_of(&self, table: &str) -> &[FieldInfo] {
		self.tables
			.get(table)
			.map(|t| t.fields.as_slice())
			.unwrap_or(&[])
	}

	/// Indexes defined on a table.
	pub fn indexes_of(&self, table: &str) -> &[IndexInfo] {
		self.tables
			.get(table)
			.map(|t| t.indexes.as_slice())
			.unwrap_or(&[])
	}

	/// Events defined on a table.
	pub fn events_of(&self, table: &str) -> &[EventInfo] {
		self.tables
			.get(table)
			.map(|t| t.events.as_slice())
			.unwrap_or(&[])
	}

	/// Get function info by name (without the `fn::` prefix).
	pub fn function(&self, name: &str) -> Option<&FunctionInfo> {
		self.functions.get(name)
	}

	/// All function names (without `fn::` prefix).
	pub fn function_names(&self) -> Vec<&str> {
		self.functions.keys().map(|s| s.as_str()).collect()
	}

	/// All defined param names.
	pub fn param_names(&self) -> Vec<&str> {
		self.params.keys().map(|s| s.as_str()).collect()
	}
}

// ─── Helpers ───

fn expr_to_string(expr: &crate::Expr) -> String {
	let mut s = String::new();
	expr.fmt_sql(&mut s, SqlFormat::SingleLine);
	// Strip backtick/bracket escaping for clean lookup keys
	s.trim_matches('`')
		.trim_matches('⟨')
		.trim_matches('⟩')
		.to_string()
}

/// Extract table names from record link types (e.g., `record<user>` → `["user"]`).
fn extract_record_links(kind: &crate::Kind) -> Vec<String> {
	let mut s = String::new();
	kind.fmt_sql(&mut s, SqlFormat::SingleLine);

	let mut links = Vec::new();
	// Parse record<table1 | table2> patterns from the formatted string
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
