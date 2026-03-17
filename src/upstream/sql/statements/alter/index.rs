use crate::upstream::fmt::{EscapeKwFreeIdent, EscapeKwIdent, QuoteStr};
use crate::upstream::sql::statements::alter::AlterKind;
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AlterIndexStatement {
	pub name: String,
	pub table: String,
	pub if_exists: bool,
	pub prepare_remove: bool,
	pub comment: AlterKind<String>,
}
impl ToSql for AlterIndexStatement {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		write_sql!(f, fmt, "ALTER INDEX");
		if self.if_exists {
			write_sql!(f, fmt, " IF EXISTS");
		}
		write_sql!(
			f,
			fmt,
			" {} ON {}",
			EscapeKwIdent(&self.name, &["IF"]),
			EscapeKwFreeIdent(&self.table)
		);
		if self.prepare_remove {
			write_sql!(f, fmt, " PREPARE REMOVE");
		}
		match self.comment {
			AlterKind::Set(ref x) => write_sql!(f, fmt, " COMMENT {}", QuoteStr(x)),
			AlterKind::Drop => write_sql!(f, fmt, " DROP COMMENT"),
			AlterKind::None => {}
		}
	}
}
