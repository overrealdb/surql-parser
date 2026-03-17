use crate::upstream::fmt::{CoverStmts, EscapeKwFreeIdent};
use crate::upstream::sql::{Expr, Kind};
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct SetStatement {
	pub name: String,
	pub what: Expr,
	pub kind: Option<Kind>,
}
impl ToSql for SetStatement {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		write_sql!(f, fmt, "LET ${}", EscapeKwFreeIdent(&self.name));
		if let Some(ref kind) = self.kind {
			write_sql!(f, fmt, ": {}", kind);
		}
		write_sql!(f, fmt, " = {}", CoverStmts(&self.what));
	}
}
