use crate::upstream::fmt::{EscapeKwFreeIdent, Fmt};
use crate::upstream::sql::order::Ordering;
use crate::upstream::sql::{
	Cond, Dir, Fields, Groups, Idiom, Limit, RecordIdKeyRangeLit, Splits, Start,
};
use std::ops::Bound;
use surrealdb_types::{SqlFormat, ToSql, write_sql};
/// A lookup is a unified way of looking up graph edges and record references.
/// Since they both work very similarly, they also both support the same operations
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lookup {
	pub kind: LookupKind,
	pub expr: Option<Fields>,
	pub what: Vec<LookupSubject>,
	pub cond: Option<Cond>,
	pub split: Option<Splits>,
	pub group: Option<Groups>,
	pub order: Option<Ordering>,
	pub limit: Option<Limit>,
	pub start: Option<Start>,
	pub alias: Option<Idiom>,
}
impl ToSql for Lookup {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		if self.what.len() <= 1
			&& self.what.iter().all(|v| {
				if v.referencing_field().is_some() {
					return false;
				}
				if let LookupSubject::Range {
					range: RecordIdKeyRangeLit {
						end: Bound::Unbounded,
						..
					},
					..
				} = v
				{
					return false;
				}
				true
			}) && self.cond.is_none()
			&& self.alias.is_none()
			&& self.expr.is_none()
		{
			self.kind.fmt_sql(f, fmt);
			if self.what.is_empty() {
				f.push('?');
			} else {
				write_sql!(f, fmt, "{}", Fmt::comma_separated(self.what.iter()));
			}
		} else {
			write_sql!(f, fmt, "{}(", self.kind);
			if let Some(ref expr) = self.expr {
				write_sql!(f, fmt, "SELECT {} FROM ", expr);
			}
			if self.what.is_empty() {
				f.push('?');
			} else {
				write_sql!(f, fmt, "{}", Fmt::comma_separated(&self.what));
			}
			if let Some(ref v) = self.cond {
				write_sql!(f, fmt, " {v}");
			}
			if let Some(ref v) = self.split {
				write_sql!(f, fmt, " {v}");
			}
			if let Some(ref v) = self.group {
				write_sql!(f, fmt, " {v}");
			}
			if let Some(ref v) = self.order {
				write_sql!(f, fmt, " {v}");
			}
			if let Some(ref v) = self.limit {
				write_sql!(f, fmt, " {v}");
			}
			if let Some(ref v) = self.start {
				write_sql!(f, fmt, " {v}");
			}
			if let Some(ref v) = self.alias {
				write_sql!(f, fmt, " AS {v}");
			}
			f.push(')');
		}
	}
}
/// This enum instructs whether the lookup is a graph edge or a record reference
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum LookupKind {
	Graph(Dir),
	Reference,
}
impl Default for LookupKind {
	fn default() -> Self {
		Self::Graph(Dir::Both)
	}
}
impl ToSql for LookupKind {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		match self {
			Self::Graph(dir) => dir.fmt_sql(f, fmt),
			Self::Reference => f.push_str("<~"),
		}
	}
}
/// This enum instructs whether we scan all edges on a table or just a specific range
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum LookupSubject {
	Table {
		table: String,
		referencing_field: Option<String>,
	},
	Range {
		table: String,
		range: RecordIdKeyRangeLit,
		referencing_field: Option<String>,
	},
}
impl LookupSubject {
	pub fn referencing_field(&self) -> Option<&String> {
		match self {
			LookupSubject::Table {
				referencing_field, ..
			} => referencing_field.as_ref(),
			LookupSubject::Range {
				referencing_field, ..
			} => referencing_field.as_ref(),
		}
	}
}
impl ToSql for LookupSubject {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		match self {
			Self::Table {
				table,
				referencing_field,
			} => {
				write_sql!(f, fmt, "{}", EscapeKwFreeIdent(table));
				if let Some(referencing_field) = referencing_field {
					write_sql!(f, fmt, " FIELD {}", EscapeKwFreeIdent(referencing_field));
				}
			}
			Self::Range {
				table,
				range,
				referencing_field,
			} => {
				write_sql!(f, fmt, "{}:{range}", EscapeKwFreeIdent(table));
				if let Some(referencing_field) = referencing_field {
					write_sql!(f, fmt, " FIELD {}", EscapeKwFreeIdent(referencing_field));
				}
			}
		}
	}
}
