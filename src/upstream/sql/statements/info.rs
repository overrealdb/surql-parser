use crate::upstream::fmt::CoverStmts;
use crate::upstream::sql::{Base, Expr};
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum InfoStatement {
	Root(bool),
	Ns(bool),
	Db(bool, Option<Expr>),
	Tb(Expr, bool, Option<Expr>),
	User(Expr, Option<Base>, bool),
	Index(Expr, Expr, bool),
}
impl ToSql for InfoStatement {
	fn fmt_sql(&self, f: &mut String, sql_fmt: SqlFormat) {
		match self {
			Self::Root(false) => f.push_str("INFO FOR ROOT"),
			Self::Root(true) => f.push_str("INFO FOR ROOT STRUCTURE"),
			Self::Ns(false) => f.push_str("INFO FOR NAMESPACE"),
			Self::Ns(true) => f.push_str("INFO FOR NAMESPACE STRUCTURE"),
			Self::Db(false, v) => match v {
				Some(v) => {
					write_sql!(f, sql_fmt, "INFO FOR DATABASE VERSION {}", CoverStmts(v))
				}
				None => f.push_str("INFO FOR DATABASE"),
			},
			Self::Db(true, v) => match v {
				Some(v) => {
					write_sql!(
						f,
						sql_fmt,
						"INFO FOR DATABASE VERSION {} STRUCTURE",
						CoverStmts(v)
					)
				}
				None => f.push_str("INFO FOR DATABASE STRUCTURE"),
			},
			Self::Tb(t, false, v) => match v {
				Some(v) => {
					write_sql!(
						f,
						sql_fmt,
						"INFO FOR TABLE {} VERSION {}",
						CoverStmts(t),
						CoverStmts(v)
					)
				}
				None => write_sql!(f, sql_fmt, "INFO FOR TABLE {}", CoverStmts(t)),
			},
			Self::Tb(t, true, v) => match v {
				Some(v) => {
					write_sql!(
						f,
						sql_fmt,
						"INFO FOR TABLE {} VERSION {} STRUCTURE",
						CoverStmts(t),
						CoverStmts(v)
					)
				}
				None => {
					write_sql!(f, sql_fmt, "INFO FOR TABLE {} STRUCTURE", CoverStmts(t))
				}
			},
			Self::User(u, b, false) => match b {
				Some(b) => {
					write_sql!(f, sql_fmt, "INFO FOR USER {} ON {b}", CoverStmts(u))
				}
				None => write_sql!(f, sql_fmt, "INFO FOR USER {}", CoverStmts(u)),
			},
			Self::User(u, b, true) => match b {
				Some(b) => {
					write_sql!(
						f,
						sql_fmt,
						"INFO FOR USER {} ON {b} STRUCTURE",
						CoverStmts(u)
					)
				}
				None => {
					write_sql!(f, sql_fmt, "INFO FOR USER {} STRUCTURE", CoverStmts(u))
				}
			},
			Self::Index(i, t, false) => {
				write_sql!(
					f,
					sql_fmt,
					"INFO FOR INDEX {} ON {}",
					CoverStmts(i),
					CoverStmts(t)
				)
			}
			Self::Index(i, t, true) => {
				write_sql!(
					f,
					sql_fmt,
					"INFO FOR INDEX {} ON {} STRUCTURE",
					CoverStmts(i),
					CoverStmts(t)
				)
			}
		}
	}
}
