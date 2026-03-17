use super::DefineKind;
use crate::upstream::fmt::{CoverStmts, EscapeIdent};
use crate::upstream::sql::{Expr, Literal, Permission};
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct DefineModelStatement {
	pub kind: DefineKind,
	pub hash: String,
	pub name: String,
	pub version: String,
	pub comment: Expr,
	pub permissions: Permission,
}
impl Default for DefineModelStatement {
	fn default() -> Self {
		Self {
			kind: DefineKind::Default,
			hash: String::new(),
			name: String::new(),
			version: String::new(),
			comment: Expr::Literal(Literal::None),
			permissions: Permission::default(),
		}
	}
}
impl ToSql for DefineModelStatement {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		write_sql!(f, fmt, "DEFINE MODEL");
		match self.kind {
			DefineKind::Default => {}
			DefineKind::Overwrite => write_sql!(f, fmt, " OVERWRITE"),
			DefineKind::IfNotExists => write_sql!(f, fmt, " IF NOT EXISTS"),
		}
		write_sql!(f, fmt, " ml::{}<{}>", EscapeIdent(&self.name), self.version);
		if !matches!(self.comment, Expr::Literal(Literal::None)) {
			write_sql!(f, fmt, " COMMENT {}", CoverStmts(&self.comment));
		}
		write_sql!(f, fmt, " PERMISSIONS {}", self.permissions);
	}
}
