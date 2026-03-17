use crate::upstream::fmt::CoverStmts;
use crate::upstream::sql::Expr;
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Limit(pub Expr);
impl ToSql for Limit {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		write_sql!(f, fmt, "LIMIT {}", CoverStmts(&self.0))
	}
}
