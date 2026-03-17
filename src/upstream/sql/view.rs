use crate::compat::val::TableName;
use crate::upstream::fmt::{EscapeKwFreeIdent, Fmt};
use crate::upstream::sql::{Cond, Fields, Groups};
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct View {
	pub expr: Fields,
	pub what: Vec<String>,
	pub cond: Option<Cond>,
	pub group: Option<Groups>,
}
impl ToSql for View {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		write_sql!(
			f,
			fmt,
			"AS SELECT {} FROM {}",
			self.expr,
			Fmt::comma_separated(self.what.iter().map(|x| EscapeKwFreeIdent(x.as_ref())))
		);
		if let Some(ref v) = self.cond {
			write_sql!(f, fmt, " {v}");
		}
		if let Some(ref v) = self.group {
			write_sql!(f, fmt, " {v}");
		}
	}
}
