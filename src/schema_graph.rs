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

use std::collections::{HashMap, HashSet, VecDeque};
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

/// A node in a dependency tree, representing a table linked via `record<>` fields.
///
/// Used by [`SchemaGraph::dependency_tree`] to build a nested view of how tables
/// are connected through record links.
#[derive(Debug, Clone)]
pub struct DependencyNode {
	/// Table name at this node.
	pub table: String,
	/// The field on the *parent* node that links to this table (`None` for the root).
	pub field: Option<String>,
	/// Child nodes — tables linked from this node's `record<>` fields.
	pub children: Vec<DependencyNode>,
	/// `true` if this table was already visited in an ancestor node (cycle detected).
	pub is_cycle: bool,
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
	pub fn from_source(input: &str) -> crate::Result<Self> {
		let defs = crate::extract_definitions(input)?;
		Ok(Self::from_definitions(&defs))
	}

	/// Build a schema graph by walking all `.surql` files in a directory.
	///
	/// Source locations are tracked per file for go-to-definition support.
	pub fn from_files(dir: &Path) -> anyhow::Result<Self> {
		let mut graph = Self::default();
		Self::collect_files_recursive(dir, &mut graph, 0)?;
		Ok(graph)
	}

	/// Build per-file schema graphs by walking all `.surql` files in a directory.
	///
	/// Returns a map from file path to its individual `SchemaGraph`.
	/// Used for incremental rebuilding: on save, only the changed file is re-parsed,
	/// then all per-file graphs are merged.
	pub fn from_files_per_file(dir: &Path) -> anyhow::Result<HashMap<std::path::PathBuf, Self>> {
		let mut per_file = HashMap::new();
		Self::collect_files_per_file_recursive(dir, &mut per_file, 0)?;
		Ok(per_file)
	}

	/// Build a schema graph from a single `.surql` file.
	///
	/// Returns `None` if the file cannot be read or parsed.
	pub fn from_single_file(path: &Path) -> Option<Self> {
		let content = match crate::read_surql_file(path) {
			Ok(c) => c,
			Err(e) => {
				tracing::warn!("{e}");
				return None;
			}
		};
		let (stmts, _) = crate::parse_with_recovery(&content);
		match crate::extract_definitions_from_ast(&stmts) {
			Ok(defs) => {
				let mut graph = Self::from_definitions(&defs);
				graph.attach_source_locations(&content, path);
				Some(graph)
			}
			Err(e) => {
				tracing::warn!("Cannot extract defs from {}: {e}", path.display());
				None
			}
		}
	}

	fn collect_files_per_file_recursive(
		dir: &Path,
		per_file: &mut HashMap<std::path::PathBuf, Self>,
		depth: u32,
	) -> anyhow::Result<()> {
		if depth > 32 {
			tracing::warn!(
				"Max directory depth (32) exceeded at {}, skipping",
				dir.display()
			);
			return Ok(());
		}
		let mut entries: Vec<_> = std::fs::read_dir(dir)?
			.filter_map(|e| match e {
				Ok(entry) => Some(entry),
				Err(err) => {
					tracing::warn!("Skipping unreadable entry in {}: {err}", dir.display());
					None
				}
			})
			.collect();
		entries.sort_by_key(|e| e.file_name());

		for entry in entries {
			let path = entry.path();
			if path
				.symlink_metadata()
				.map(|m| m.is_symlink())
				.unwrap_or(false)
			{
				tracing::warn!("Skipping symlink: {}", path.display());
				continue;
			}
			if path.is_dir() {
				let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
				if matches!(
					name,
					"target"
						| "node_modules" | ".git"
						| "build" | "fixtures"
						| "dist" | ".cache"
						| "surql-lsp-out"
				) || name.starts_with('.')
				{
					continue;
				}
			}
			if path.is_file() && path.extension().is_some_and(|ext| ext == "surql") {
				if let Some(graph) = Self::from_single_file(&path) {
					per_file.insert(path, graph);
				}
			} else if path.is_dir() {
				Self::collect_files_per_file_recursive(&path, per_file, depth + 1)?;
			}
		}
		Ok(())
	}

	fn collect_files_recursive(dir: &Path, graph: &mut Self, depth: u32) -> anyhow::Result<()> {
		if depth > 32 {
			tracing::warn!(
				"Max directory depth (32) exceeded at {}, skipping",
				dir.display()
			);
			return Ok(());
		}
		let mut entries: Vec<_> = std::fs::read_dir(dir)?
			.filter_map(|e| match e {
				Ok(entry) => Some(entry),
				Err(err) => {
					tracing::warn!("Skipping unreadable entry in {}: {err}", dir.display());
					None
				}
			})
			.collect();
		entries.sort_by_key(|e| e.file_name());

		for entry in entries {
			let path = entry.path();
			if path
				.symlink_metadata()
				.map(|m| m.is_symlink())
				.unwrap_or(false)
			{
				tracing::warn!("Skipping symlink: {}", path.display());
				continue;
			}
			if path.is_dir() {
				let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
				if matches!(
					name,
					"target"
						| "node_modules" | ".git"
						| "build" | "fixtures"
						| "dist" | ".cache"
						| "surql-lsp-out"
				) || name.starts_with('.')
				{
					continue;
				}
			}
			if path.is_file() && path.extension().is_some_and(|ext| ext == "surql") {
				let content = match crate::read_surql_file(&path) {
					Ok(c) => c,
					Err(e) => {
						tracing::warn!("Skipping {e}");
						continue;
					}
				};
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
				Self::collect_files_recursive(&path, graph, depth + 1)?;
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
				let existing_field_names: HashSet<String> =
					existing.fields.iter().map(|f| f.name.clone()).collect();
				for field in table.fields {
					if !existing_field_names.contains(&field.name) {
						existing.fields.push(field);
					}
				}
				let existing_index_names: HashSet<String> =
					existing.indexes.iter().map(|i| i.name.clone()).collect();
				for index in table.indexes {
					if !existing_index_names.contains(&index.name) {
						existing.indexes.push(index);
					}
				}
				let existing_event_names: HashSet<String> =
					existing.events.iter().map(|e| e.name.clone()).collect();
				for event in table.events {
					if !existing_event_names.contains(&event.name) {
						existing.events.push(event);
					}
				}
				if table.source.is_some() {
					existing.source = table.source;
				}
			} else {
				self.tables.insert(name, table);
			}
		}
		for (name, func) in other.functions {
			if let std::collections::hash_map::Entry::Vacant(e) = self.functions.entry(name.clone())
			{
				e.insert(func);
			} else {
				tracing::warn!("Duplicate function definition: fn::{name} (keeping first)");
			}
		}
		for (name, param) in other.params {
			if let std::collections::hash_map::Entry::Vacant(e) = self.params.entry(name.clone()) {
				e.insert(param);
			} else {
				tracing::warn!("Duplicate param definition: ${name} (keeping first)");
			}
		}
		self.rebuild_field_index();
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
				if !table.fields.iter().any(|f| f.name == def.name) {
					table.fields.push(def);
				}
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

	/// Return a filtered schema containing only tables matching the given NS/DB scope.
	/// Tables with no NS/DB (None) are included in all scopes (default context).
	pub fn scoped(&self, ns: Option<&str>, db: Option<&str>) -> Self {
		let tables: HashMap<String, TableDef> = self
			.tables
			.iter()
			.filter(|(_, t)| scope_matches(&t.ns, &t.db, ns, db))
			.map(|(k, v)| (k.clone(), v.clone()))
			.collect();
		let mut graph = Self {
			tables,
			functions: self.functions.clone(),
			params: self.params.clone(),
			field_index: HashMap::new(),
		};
		graph.rebuild_field_index();
		graph
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

	// ─── Graph Traversal ───

	/// Tables reachable from `start` within `max_depth` hops via `record<>` links.
	///
	/// Uses BFS with cycle detection. Returns `(table_name, depth, path)` where
	/// `path` is the chain of `table.field` segments traversed to reach each table.
	///
	/// The starting table itself is not included in the results.
	pub fn tables_reachable_from(
		&self,
		start: &str,
		max_depth: usize,
	) -> Vec<(String, usize, Vec<String>)> {
		let mut results = Vec::new();
		if !self.tables.contains_key(start) {
			return results;
		}

		// (current_table, depth, path_so_far)
		let mut queue: VecDeque<(String, usize, Vec<String>)> = VecDeque::new();
		let mut visited = HashSet::new();
		visited.insert(start.to_string());
		queue.push_back((start.to_string(), 0, Vec::new()));

		while let Some((current, depth, path)) = queue.pop_front() {
			if depth >= max_depth {
				continue;
			}
			let Some(table) = self.tables.get(&current) else {
				continue;
			};
			for field in &table.fields {
				for link in &field.record_links {
					let mut new_path = path.clone();
					new_path.push(format!("{current}.{}", field.name));
					if visited.insert(link.clone()) {
						results.push((link.clone(), depth + 1, new_path.clone()));
						queue.push_back((link.clone(), depth + 1, new_path));
					}
				}
			}
		}

		results.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
		results
	}

	/// Tables that reference `target` via `record<target>` fields (reverse dependencies).
	///
	/// Returns `(referencing_table, field_name)` pairs. Useful for answering:
	/// "What would break if I drop or rename this table?"
	pub fn tables_referencing(&self, target: &str) -> Vec<(String, String)> {
		let mut results = Vec::new();
		let mut table_names: Vec<&str> = self.tables.keys().map(|s| s.as_str()).collect();
		table_names.sort();
		for table_name in table_names {
			let Some(table) = self.tables.get(table_name) else {
				continue;
			};
			for field in &table.fields {
				if field.record_links.iter().any(|l| l == target) {
					results.push((table_name.to_string(), field.name.clone()));
				}
			}
		}
		results
	}

	/// Tables that share a common `record<>` target with `table` (siblings).
	///
	/// Answers: "Which other tables also link to the same targets as this table?"
	/// Returns `(sibling_table, shared_target, via_field)` triples.
	pub fn siblings_of(&self, table: &str) -> Vec<(String, String, String)> {
		let Some(source_table) = self.tables.get(table) else {
			return Vec::new();
		};

		let targets: HashSet<&str> = source_table
			.fields
			.iter()
			.flat_map(|f| f.record_links.iter().map(|l| l.as_str()))
			.collect();

		if targets.is_empty() {
			return Vec::new();
		}

		let mut results = Vec::new();
		let mut other_names: Vec<&str> = self
			.tables
			.keys()
			.filter(|n| n.as_str() != table)
			.map(|s| s.as_str())
			.collect();
		other_names.sort();

		for other_name in other_names {
			let Some(other_table) = self.tables.get(other_name) else {
				continue;
			};
			for field in &other_table.fields {
				for link in &field.record_links {
					if targets.contains(link.as_str()) {
						results.push((other_name.to_string(), link.clone(), field.name.clone()));
					}
				}
			}
		}
		results
	}

	/// Build a full dependency tree rooted at `root`, following `record<>` links.
	///
	/// Cycles are detected: if a table appears as its own ancestor, the node
	/// is marked with `is_cycle = true` and has no children (the recursion stops).
	///
	/// A `max_nodes` safety cap (1000) prevents exponential blowup on diamond-shaped
	/// graphs where the same table is reachable through many paths.
	pub fn dependency_tree(&self, root: &str, max_depth: usize) -> DependencyNode {
		let mut visited = HashSet::new();
		let mut node_count = 0usize;
		self.build_dependency_subtree(root, None, max_depth, &mut visited, &mut node_count)
	}

	fn build_dependency_subtree(
		&self,
		table_name: &str,
		via_field: Option<&str>,
		remaining_depth: usize,
		visited: &mut HashSet<String>,
		node_count: &mut usize,
	) -> DependencyNode {
		*node_count += 1;

		if !visited.insert(table_name.to_string()) {
			return DependencyNode {
				table: table_name.to_string(),
				field: via_field.map(|s| s.to_string()),
				children: Vec::new(),
				is_cycle: true,
			};
		}

		let mut children = Vec::new();
		if remaining_depth > 0
			&& *node_count < 1000
			&& let Some(table) = self.tables.get(table_name)
		{
			let mut field_links: Vec<(&str, &str)> = Vec::new();
			for field in &table.fields {
				for link in &field.record_links {
					field_links.push((field.name.as_str(), link.as_str()));
				}
			}
			field_links.sort_by(|a, b| a.0.cmp(b.0).then_with(|| a.1.cmp(b.1)));

			for (field_name, link_target) in field_links {
				if *node_count >= 1000 {
					break;
				}
				let child = self.build_dependency_subtree(
					link_target,
					Some(field_name),
					remaining_depth - 1,
					visited,
					node_count,
				);
				children.push(child);
			}
		}

		visited.remove(table_name);

		DependencyNode {
			table: table_name.to_string(),
			field: via_field.map(|s| s.to_string()),
			children,
			is_cycle: false,
		}
	}

	/// Generate a markdown graph tree for all tables in the schema.
	///
	/// Builds a dependency tree for each table and formats it as an
	/// indented markdown tree view. Used by the LSP to cache `graph.md`.
	pub fn build_graph_tree_markdown(&self) -> String {
		let mut table_names: Vec<&str> = self.table_names().collect();
		table_names.sort();

		if table_names.is_empty() {
			return "# Schema Graph\n\nNo tables defined.\n".to_string();
		}

		let mut out = String::from("# Schema Graph\n\n");

		// Summary: tables with outgoing links
		let mut link_count = 0usize;
		for name in &table_names {
			if let Some(table) = self.tables.get(*name) {
				for field in &table.fields {
					link_count += field.record_links.len();
				}
			}
		}
		out.push_str(&format!(
			"**{} tables, {} record links**\n\n",
			table_names.len(),
			link_count,
		));

		// Dependency tree per table (only tables that have outgoing or incoming links)
		for name in &table_names {
			let tree = self.dependency_tree(name, 5);
			if tree.children.is_empty() && self.tables_referencing(name).is_empty() {
				continue;
			}
			out.push_str(&format!("## {name}\n\n"));

			// Forward deps
			if !tree.children.is_empty() {
				out.push_str("**Depends on:**\n```\n");
				for child in &tree.children {
					format_dependency_node(&mut out, child, 0);
				}
				out.push_str("```\n\n");
			}

			// Reverse deps
			let refs = self.tables_referencing(name);
			if !refs.is_empty() {
				out.push_str("**Referenced by:**\n");
				for (ref_table, ref_field) in &refs {
					out.push_str(&format!("- `{ref_table}.{ref_field}`\n"));
				}
				out.push('\n');
			}

			// Siblings
			let siblings = self.siblings_of(name);
			if !siblings.is_empty() {
				out.push_str("**Siblings (shared targets):**\n");
				let mut seen = HashSet::new();
				for (sib, target, field) in &siblings {
					let key = format!("{sib}.{field}->{target}");
					if seen.insert(key) {
						out.push_str(&format!(
							"- `{sib}` (both link to `{target}` via `.{field}`)\n"
						));
					}
				}
				out.push('\n');
			}
		}

		out
	}

	/// Generate markdown documentation from schema definitions.
	///
	/// Includes tables with fields (type, default, comment), indexes, events,
	/// and function signatures with parameters and return types.
	pub fn build_docs_markdown(&self) -> String {
		let mut out = String::from("# Schema Documentation\n\n");

		let mut table_names: Vec<&str> = self.table_names().collect();
		table_names.sort();

		if !table_names.is_empty() {
			out.push_str("## Tables\n\n");
			for name in &table_names {
				let table = match self.table(name) {
					Some(t) => t,
					None => continue,
				};
				out.push_str(&format!("### {name}\n\n"));
				if let Some(comment) = &table.comment {
					out.push_str(comment);
					out.push_str("\n\n");
				}

				let schema_label = if table.full {
					"SCHEMAFULL"
				} else {
					"SCHEMALESS"
				};
				out.push_str(&format!("*{schema_label}*\n\n"));

				if !table.fields.is_empty() {
					out.push_str("| Field | Type | Default | Comment |\n");
					out.push_str("|-------|------|---------|--------|\n");
					for field in &table.fields {
						let kind = field.kind.as_deref().unwrap_or("");
						let default = field.default.as_deref().unwrap_or("");
						let comment = field.comment.as_deref().unwrap_or("");
						out.push_str(&format!(
							"| {} | {} | {} | {} |\n",
							field.name,
							escape_markdown_table(kind),
							escape_markdown_table(default),
							escape_markdown_table(comment),
						));
					}
					out.push('\n');
				}

				if !table.indexes.is_empty() {
					out.push_str("**Indexes:**\n");
					for idx in &table.indexes {
						let unique_label = if idx.unique { " (UNIQUE)" } else { "" };
						let cols = idx.columns.join(", ");
						out.push_str(&format!("- `{}`{unique_label} on `{cols}`\n", idx.name));
					}
					out.push('\n');
				}

				if !table.events.is_empty() {
					out.push_str("**Events:**\n");
					for ev in &table.events {
						let comment_suffix = ev
							.comment
							.as_ref()
							.map(|c| format!(" -- {c}"))
							.unwrap_or_default();
						out.push_str(&format!("- `{}`{comment_suffix}\n", ev.name));
					}
					out.push('\n');
				}
			}
		}

		let mut fn_names: Vec<&str> = self.function_names().collect();
		fn_names.sort();

		if !fn_names.is_empty() {
			out.push_str("## Functions\n\n");
			for name in &fn_names {
				let func = match self.function(name) {
					Some(f) => f,
					None => continue,
				};
				out.push_str(&format!("### fn::{name}\n\n"));
				if let Some(comment) = &func.comment {
					out.push_str(comment);
					out.push_str("\n\n");
				}

				if !func.args.is_empty() {
					let args_str: Vec<String> = func
						.args
						.iter()
						.map(|(n, k)| {
							let name = n.strip_prefix('$').unwrap_or(n);
							format!("${name}: {k}")
						})
						.collect();
					out.push_str(&format!("**Parameters:** `{}`\n", args_str.join(", ")));
				}

				if let Some(ret) = &func.returns {
					out.push_str(&format!("**Returns:** `{ret}`\n"));
				}

				out.push('\n');
			}
		}

		out
	}
}

fn escape_markdown_table(s: &str) -> String {
	s.replace('|', "\\|")
}

fn format_dependency_node(out: &mut String, node: &DependencyNode, indent: usize) {
	let prefix = "  ".repeat(indent);
	let field_label = node
		.field
		.as_deref()
		.map(|f| format!(".{f} -> "))
		.unwrap_or_default();
	let cycle_label = if node.is_cycle { " (cycle)" } else { "" };
	out.push_str(&format!(
		"{prefix}{field_label}[{}]{cycle_label}\n",
		node.table
	));
	if !node.is_cycle {
		for child in &node.children {
			format_dependency_node(out, child, indent + 1);
		}
	}
}

// ─── Token & AST Extraction ───

fn token_text<'a>(source: &'a str, token: &crate::upstream::syn::token::Token) -> &'a str {
	let start = token.span.offset as usize;
	let end = (token.span.offset + token.span.len) as usize;
	if end <= source.len() {
		&source[start..end]
	} else {
		tracing::warn!(
			"Token span [{start}..{end}] exceeds source length {}, returning empty",
			source.len()
		);
		""
	}
}

fn expr_to_string(expr: &crate::Expr) -> String {
	let mut s = String::new();
	expr.fmt_sql(&mut s, SqlFormat::SingleLine);
	let s = s.strip_prefix('`').unwrap_or(&s);
	let s = s.strip_suffix('`').unwrap_or(s);
	let s = s.strip_prefix('\u{27E8}').unwrap_or(s);
	let s = s.strip_suffix('\u{27E9}').unwrap_or(s);
	s.to_string()
}

/// Check if a table's NS/DB matches the requested scope.
/// Tables with no NS/DB (None) match any scope (they're in the default context).
fn scope_matches(
	table_ns: &Option<String>,
	table_db: &Option<String>,
	filter_ns: Option<&str>,
	filter_db: Option<&str>,
) -> bool {
	let ns_ok = match (table_ns, filter_ns) {
		(None, _) => true,
		(Some(t), Some(f)) => t.eq_ignore_ascii_case(f),
		(Some(_), None) => true,
	};
	let db_ok = match (table_db, filter_db) {
		(None, _) => true,
		(Some(t), Some(f)) => t.eq_ignore_ascii_case(f),
		(Some(_), None) => true,
	};
	ns_ok && db_ok
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
