use crate::upstream::fmt::EscapeIdent;
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct RemoveModelStatement {
	pub name: String,
	pub version: String,
	pub if_exists: bool,
}
impl ToSql for RemoveModelStatement {
	fn fmt_sql(&self, f: &mut String, sql_fmt: SqlFormat) {
		write_sql!(f, sql_fmt, "REMOVE MODEL");
		if self.if_exists {
			write_sql!(f, sql_fmt, " IF EXISTS");
		}
		write_sql!(
			f,
			sql_fmt,
			" ml::{}<{}>",
			EscapeIdent(&self.name),
			self.version
		);
	}
}
