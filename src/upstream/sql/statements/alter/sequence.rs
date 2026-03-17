use crate::upstream::fmt::{CoverStmts, EscapeKwIdent};
use crate::upstream::sql::Expr;
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Default)]
pub struct AlterSequenceStatement {
	pub name: String,
	pub if_exists: bool,
	pub timeout: Option<Expr>,
}
impl ToSql for AlterSequenceStatement {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		write_sql!(f, fmt, "ALTER SEQUENCE");
		if self.if_exists {
			write_sql!(f, fmt, " IF EXISTS");
		}
		write_sql!(f, fmt, " {}", EscapeKwIdent(&self.name, &["IF"]));
		if let Some(timeout) = &self.timeout {
			write_sql!(f, fmt, " TIMEOUT {}", CoverStmts(timeout));
		}
	}
}
