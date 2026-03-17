use crate::upstream::fmt::CoverStmts;
use crate::upstream::sql::Expr;
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct KillStatement {
	pub id: Expr,
}
impl ToSql for KillStatement {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		write_sql!(f, fmt, "KILL {}", CoverStmts(&self.id));
	}
}
