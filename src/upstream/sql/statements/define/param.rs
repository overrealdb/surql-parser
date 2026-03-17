use super::DefineKind;
use crate::upstream::fmt::{CoverStmts, EscapeKwFreeIdent};
use crate::upstream::sql::{Expr, Literal, Permission};
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct DefineParamStatement {
	pub kind: DefineKind,
	pub name: String,
	pub value: Expr,
	pub comment: Expr,
	pub permissions: Permission,
}
impl ToSql for DefineParamStatement {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		write_sql!(f, fmt, "DEFINE PARAM");
		match self.kind {
			DefineKind::Default => {}
			DefineKind::Overwrite => write_sql!(f, fmt, " OVERWRITE"),
			DefineKind::IfNotExists => write_sql!(f, fmt, " IF NOT EXISTS"),
		}
		write_sql!(
			f,
			fmt,
			" ${} VALUE {}",
			EscapeKwFreeIdent(&self.name),
			CoverStmts(&self.value)
		);
		if !matches!(self.comment, Expr::Literal(Literal::None)) {
			write_sql!(f, fmt, " COMMENT {}", CoverStmts(&self.comment));
		}
		let fmt = fmt.increment();
		write_sql!(f, fmt, " PERMISSIONS {}", self.permissions);
	}
}
