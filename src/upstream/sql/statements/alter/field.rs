use super::AlterKind;
use crate::upstream::fmt::{CoverStmts, EscapeKwFreeIdent, QuoteStr};
use crate::upstream::sql::reference::Reference;
use crate::upstream::sql::{Expr, Idiom, Kind, Permissions};
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum AlterDefault {
	#[default]
	None,
	Drop,
	Always(Expr),
	Set(Expr),
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct AlterFieldStatement {
	#[cfg_attr(
        feature = "arbitrary",
        arbitrary(with = crate::upstream::sql::arbitrary::local_idiom)
    )]
	pub name: Idiom,
	pub what: String,
	pub if_exists: bool,
	pub kind: AlterKind<Kind>,
	pub flexible: AlterKind<()>,
	pub readonly: AlterKind<()>,
	pub value: AlterKind<Expr>,
	pub assert: AlterKind<Expr>,
	pub default: AlterDefault,
	pub permissions: Option<Permissions>,
	pub comment: AlterKind<String>,
	pub reference: AlterKind<Reference>,
}
impl ToSql for AlterFieldStatement {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		write_sql!(f, fmt, "ALTER FIELD");
		if self.if_exists {
			write_sql!(f, fmt, " IF EXISTS");
		}
		write_sql!(
			f,
			fmt,
			" {} ON {}",
			self.name,
			EscapeKwFreeIdent(&self.what)
		);
		match self.kind {
			AlterKind::Set(ref x) => write_sql!(f, fmt, " TYPE {x}"),
			AlterKind::Drop => write_sql!(f, fmt, " DROP TYPE"),
			AlterKind::None => {}
		}
		match self.flexible {
			AlterKind::Set(_) => write_sql!(f, fmt, " FLEXIBLE"),
			AlterKind::Drop => write_sql!(f, fmt, " DROP FLEXIBLE"),
			AlterKind::None => {}
		}
		match self.readonly {
			AlterKind::Set(_) => write_sql!(f, fmt, " READONLY"),
			AlterKind::Drop => write_sql!(f, fmt, " DROP READONLY"),
			AlterKind::None => {}
		}
		match self.value {
			AlterKind::Set(ref x) => write_sql!(f, fmt, " VALUE {}", CoverStmts(x)),
			AlterKind::Drop => write_sql!(f, fmt, " DROP VALUE"),
			AlterKind::None => {}
		}
		match self.assert {
			AlterKind::Set(ref x) => write_sql!(f, fmt, " ASSERT {}", CoverStmts(x)),
			AlterKind::Drop => write_sql!(f, fmt, " DROP ASSERT"),
			AlterKind::None => {}
		}
		match self.default {
			AlterDefault::None => {}
			AlterDefault::Drop => write_sql!(f, fmt, " DROP DEFAULT"),
			AlterDefault::Always(ref d) => {
				write_sql!(f, fmt, " DEFAULT ALWAYS {}", CoverStmts(d))
			}
			AlterDefault::Set(ref d) => write_sql!(f, fmt, " DEFAULT {}", CoverStmts(d)),
		}
		if let Some(permissions) = &self.permissions {
			write_sql!(f, fmt, " {permissions}");
		}
		match self.comment {
			AlterKind::Set(ref x) => write_sql!(f, fmt, " COMMENT {}", QuoteStr(x)),
			AlterKind::Drop => write_sql!(f, fmt, " DROP COMMENT"),
			AlterKind::None => {}
		}
		match self.reference {
			AlterKind::Set(ref x) => write_sql!(f, fmt, " REFERENCE {x}"),
			AlterKind::Drop => write_sql!(f, fmt, " DROP REFERENCE"),
			AlterKind::None => {}
		}
	}
}
