use super::DefineKind;
use crate::upstream::fmt::{CoverStmts, EscapeKwFreeIdent, QuoteStr};
use crate::upstream::sql::{Base, Expr, Literal};
use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, SaltString};
use rand::Rng;
use rand::distr::Alphanumeric;
use rand::rngs::OsRng;
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum PassType {
	#[default]
	Unset,
	Hash(String),
	Password(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefineUserStatement {
	pub kind: DefineKind,
	pub name: Expr,
	pub base: Base,
	pub pass_type: PassType,
	pub roles: Vec<String>,
	pub token_duration: Expr,
	pub session_duration: Expr,
	pub comment: Expr,
}
impl Default for DefineUserStatement {
	fn default() -> Self {
		Self {
			kind: DefineKind::Default,
			name: Expr::Literal(Literal::None),
			base: Base::Root,
			pass_type: PassType::Unset,
			roles: vec![],
			token_duration: Expr::Literal(Literal::None),
			session_duration: Expr::Literal(Literal::None),
			comment: Expr::Literal(Literal::None),
		}
	}
}
impl ToSql for DefineUserStatement {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		write_sql!(f, fmt, "DEFINE USER");
		match self.kind {
			DefineKind::Default => {}
			DefineKind::Overwrite => write_sql!(f, fmt, " OVERWRITE"),
			DefineKind::IfNotExists => write_sql!(f, fmt, " IF NOT EXISTS"),
		}
		write_sql!(f, fmt, " {} ON {}", CoverStmts(&self.name), &self.base);
		match self.pass_type {
			PassType::Unset => {}
			PassType::Hash(ref x) => write_sql!(f, fmt, " PASSHASH {}", QuoteStr(x)),
			PassType::Password(ref x) => write_sql!(f, fmt, " PASSWORD {}", QuoteStr(x)),
		}
		write_sql!(f, fmt, " ROLES ");
		for (idx, r) in self.roles.iter().enumerate() {
			if idx != 0 {
				f.push_str(", ");
			}
			let r = r.to_uppercase();
			EscapeKwFreeIdent(&r).fmt_sql(f, fmt);
		}
		f.push_str(" DURATION FOR TOKEN ");
		CoverStmts(&self.token_duration).fmt_sql(f, fmt);
		f.push_str(", FOR SESSION ");
		CoverStmts(&self.session_duration).fmt_sql(f, fmt);
		if !matches!(self.comment, Expr::Literal(Literal::None)) {
			write_sql!(f, fmt, " COMMENT {}", CoverStmts(&self.comment));
		}
	}
}
