//! Schema diffing — compare two SchemaGraphs and report structural changes.

use crate::SchemaGraph;
use std::collections::BTreeSet;

/// Structural diff between two schema states.
///
/// Captures added/removed tables, fields, indexes, events, and functions.
/// Changed table modes (e.g., SCHEMAFULL to SCHEMALESS) and field type changes
/// are tracked separately.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct SchemaDiff {
	pub added_tables: Vec<String>,
	pub removed_tables: Vec<String>,
	pub changed_tables: Vec<TableChange>,
	pub added_fields: Vec<(String, String)>,
	pub removed_fields: Vec<(String, String)>,
	pub changed_fields: Vec<FieldTypeChange>,
	pub added_indexes: Vec<(String, String)>,
	pub removed_indexes: Vec<(String, String)>,
	pub added_events: Vec<(String, String)>,
	pub removed_events: Vec<(String, String)>,
	pub added_functions: Vec<String>,
	pub removed_functions: Vec<String>,
}

/// A table whose definition changed between before and after.
#[derive(Debug, PartialEq, Eq)]
pub struct TableChange {
	pub name: String,
	pub before_full: bool,
	pub after_full: bool,
}

/// A field whose type changed between before and after.
#[derive(Debug, PartialEq, Eq)]
pub struct FieldTypeChange {
	pub table: String,
	pub field: String,
	pub before_type: String,
	pub after_type: String,
}

impl SchemaDiff {
	pub fn is_empty(&self) -> bool {
		self.added_tables.is_empty()
			&& self.removed_tables.is_empty()
			&& self.changed_tables.is_empty()
			&& self.added_fields.is_empty()
			&& self.removed_fields.is_empty()
			&& self.changed_fields.is_empty()
			&& self.added_indexes.is_empty()
			&& self.removed_indexes.is_empty()
			&& self.added_events.is_empty()
			&& self.removed_events.is_empty()
			&& self.added_functions.is_empty()
			&& self.removed_functions.is_empty()
	}
}

impl std::fmt::Display for SchemaDiff {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.is_empty() {
			return write!(f, "No schema changes.");
		}

		for t in &self.added_tables {
			writeln!(f, "+ TABLE {t}")?;
		}
		for t in &self.removed_tables {
			writeln!(f, "- TABLE {t}")?;
		}
		for c in &self.changed_tables {
			let before = if c.before_full {
				"SCHEMAFULL"
			} else {
				"SCHEMALESS"
			};
			let after = if c.after_full {
				"SCHEMAFULL"
			} else {
				"SCHEMALESS"
			};
			writeln!(f, "~ TABLE {}: {before} -> {after}", c.name)?;
		}
		for (table, field) in &self.added_fields {
			writeln!(f, "+ FIELD {field} ON {table}")?;
		}
		for (table, field) in &self.removed_fields {
			writeln!(f, "- FIELD {field} ON {table}")?;
		}
		for change in &self.changed_fields {
			writeln!(
				f,
				"~ FIELD {} ON {}: {} -> {}",
				change.field, change.table, change.before_type, change.after_type
			)?;
		}
		for (table, idx) in &self.added_indexes {
			writeln!(f, "+ INDEX {idx} ON {table}")?;
		}
		for (table, idx) in &self.removed_indexes {
			writeln!(f, "- INDEX {idx} ON {table}")?;
		}
		for (table, ev) in &self.added_events {
			writeln!(f, "+ EVENT {ev} ON {table}")?;
		}
		for (table, ev) in &self.removed_events {
			writeln!(f, "- EVENT {ev} ON {table}")?;
		}
		for func in &self.added_functions {
			writeln!(f, "+ FUNCTION fn::{func}")?;
		}
		for func in &self.removed_functions {
			writeln!(f, "- FUNCTION fn::{func}")?;
		}
		Ok(())
	}
}

/// Compare two schema graphs and return a structured diff.
///
/// # Example
///
/// ```
/// use surql_parser::{SchemaGraph, diff::compare_schemas};
///
/// let before = SchemaGraph::from_source("
///     DEFINE TABLE user SCHEMAFULL;
///     DEFINE FIELD name ON user TYPE string;
/// ").unwrap();
///
/// let after = SchemaGraph::from_source("
///     DEFINE TABLE user SCHEMAFULL;
///     DEFINE FIELD name ON user TYPE string;
///     DEFINE FIELD email ON user TYPE string;
///     DEFINE TABLE post SCHEMALESS;
/// ").unwrap();
///
/// let diff = compare_schemas(&before, &after);
/// assert_eq!(diff.added_tables, vec!["post"]);
/// assert_eq!(diff.added_fields, vec![("user".to_string(), "email".to_string())]);
/// ```
pub fn compare_schemas(before: &SchemaGraph, after: &SchemaGraph) -> SchemaDiff {
	let mut diff = SchemaDiff::default();

	let before_tables: BTreeSet<&str> = before.table_names().collect();
	let after_tables: BTreeSet<&str> = after.table_names().collect();

	for &name in &after_tables {
		if !before_tables.contains(name) {
			diff.added_tables.push(name.to_string());
			// Name comes from table_names() iterator over the same SchemaGraph — guaranteed to exist
			let Some(table) = after.table(name) else {
				continue;
			};
			for field in &table.fields {
				diff.added_fields
					.push((name.to_string(), field.name.clone()));
			}
			for idx in &table.indexes {
				diff.added_indexes
					.push((name.to_string(), idx.name.clone()));
			}
			for ev in &table.events {
				diff.added_events.push((name.to_string(), ev.name.clone()));
			}
		}
	}
	for &name in &before_tables {
		if !after_tables.contains(name) {
			diff.removed_tables.push(name.to_string());
			// Name comes from table_names() iterator over the same SchemaGraph — guaranteed to exist
			let Some(table) = before.table(name) else {
				continue;
			};
			for field in &table.fields {
				diff.removed_fields
					.push((name.to_string(), field.name.clone()));
			}
			for idx in &table.indexes {
				diff.removed_indexes
					.push((name.to_string(), idx.name.clone()));
			}
			for ev in &table.events {
				diff.removed_events
					.push((name.to_string(), ev.name.clone()));
			}
		}
	}

	let common_tables: BTreeSet<&str> =
		before_tables.intersection(&after_tables).copied().collect();
	for &name in &common_tables {
		// Names come from intersection of both table_names() iterators — guaranteed to exist in both
		let Some(before_table) = before.table(name) else {
			continue;
		};
		let Some(after_table) = after.table(name) else {
			continue;
		};

		if before_table.full != after_table.full {
			diff.changed_tables.push(TableChange {
				name: name.to_string(),
				before_full: before_table.full,
				after_full: after_table.full,
			});
		}

		let before_fields: BTreeSet<&str> = before_table
			.fields
			.iter()
			.map(|f| f.name.as_str())
			.collect();
		let after_fields: BTreeSet<&str> =
			after_table.fields.iter().map(|f| f.name.as_str()).collect();
		for &field in &after_fields {
			if !before_fields.contains(field) {
				diff.added_fields
					.push((name.to_string(), field.to_string()));
			}
		}
		for &field in &before_fields {
			if !after_fields.contains(field) {
				diff.removed_fields
					.push((name.to_string(), field.to_string()));
			}
		}

		// Detect field type changes for fields present in both
		for before_field in &before_table.fields {
			if let Some(after_field) = after_table
				.fields
				.iter()
				.find(|f| f.name == before_field.name)
				&& before_field.kind != after_field.kind
			{
				diff.changed_fields.push(FieldTypeChange {
					table: name.to_string(),
					field: before_field.name.clone(),
					before_type: before_field.kind.clone().unwrap_or_default(),
					after_type: after_field.kind.clone().unwrap_or_default(),
				});
			}
		}

		let before_indexes: BTreeSet<&str> = before_table
			.indexes
			.iter()
			.map(|i| i.name.as_str())
			.collect();
		let after_indexes: BTreeSet<&str> = after_table
			.indexes
			.iter()
			.map(|i| i.name.as_str())
			.collect();
		for &idx in &after_indexes {
			if !before_indexes.contains(idx) {
				diff.added_indexes.push((name.to_string(), idx.to_string()));
			}
		}
		for &idx in &before_indexes {
			if !after_indexes.contains(idx) {
				diff.removed_indexes
					.push((name.to_string(), idx.to_string()));
			}
		}

		let before_events: BTreeSet<&str> = before_table
			.events
			.iter()
			.map(|e| e.name.as_str())
			.collect();
		let after_events: BTreeSet<&str> =
			after_table.events.iter().map(|e| e.name.as_str()).collect();
		for &ev in &after_events {
			if !before_events.contains(ev) {
				diff.added_events.push((name.to_string(), ev.to_string()));
			}
		}
		for &ev in &before_events {
			if !after_events.contains(ev) {
				diff.removed_events.push((name.to_string(), ev.to_string()));
			}
		}
	}

	let before_fns: BTreeSet<&str> = before.function_names().collect();
	let after_fns: BTreeSet<&str> = after.function_names().collect();
	for &name in &after_fns {
		if !before_fns.contains(name) {
			diff.added_functions.push(name.to_string());
		}
	}
	for &name in &before_fns {
		if !after_fns.contains(name) {
			diff.removed_functions.push(name.to_string());
		}
	}

	diff
}
