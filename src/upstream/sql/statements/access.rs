use crate::compat::types::PublicDuration;
use crate::upstream::fmt::{EscapeIdent, EscapeKwFreeIdent};
use crate::upstream::sql::{Base, Cond, RecordIdLit};
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum AccessStatement {
	Grant(AccessStatementGrant),
	Show(AccessStatementShow),
	Revoke(AccessStatementRevoke),
	Purge(AccessStatementPurge),
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct AccessStatementGrant {
	pub ac: String,
	pub base: Option<Base>,
	pub subject: Subject,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct AccessStatementShow {
	pub ac: String,
	pub base: Option<Base>,
	pub gr: Option<String>,
	pub cond: Option<Cond>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct AccessStatementRevoke {
	pub ac: String,
	pub base: Option<Base>,
	pub gr: Option<String>,
	pub cond: Option<Cond>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct AccessStatementPurge {
	pub ac: String,
	pub base: Option<Base>,
	pub kind: PurgeKind,
	pub grace: PublicDuration,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum PurgeKind {
	#[default]
	Expired,
	Revoked,
	Both,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Subject {
	Record(RecordIdLit),
	User(String),
}
impl ToSql for AccessStatement {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		match self {
			Self::Grant(stmt) => {
				write_sql!(f, fmt, "ACCESS {}", EscapeKwFreeIdent(&stmt.ac));
				if let Some(ref v) = stmt.base {
					write_sql!(f, fmt, " ON {v}");
				}
				write_sql!(f, fmt, " GRANT");
				match &stmt.subject {
					Subject::User(x) => {
						write_sql!(f, fmt, " FOR USER {}", EscapeIdent(&x))
					}
					Subject::Record(x) => write_sql!(f, fmt, " FOR RECORD {}", x),
				}
			}
			Self::Show(stmt) => {
				write_sql!(f, fmt, "ACCESS {}", EscapeKwFreeIdent(&stmt.ac));
				if let Some(ref v) = stmt.base {
					write_sql!(f, fmt, " ON {v}");
				}
				write_sql!(f, fmt, " SHOW");
				match &stmt.gr {
					Some(v) => write_sql!(f, fmt, " GRANT {}", EscapeKwFreeIdent(v)),
					None => match &stmt.cond {
						Some(v) => write_sql!(f, fmt, " {v}"),
						None => write_sql!(f, fmt, " ALL"),
					},
				};
			}
			Self::Revoke(stmt) => {
				write_sql!(f, fmt, "ACCESS {}", EscapeKwFreeIdent(&stmt.ac));
				if let Some(ref v) = stmt.base {
					write_sql!(f, fmt, " ON {v}");
				}
				write_sql!(f, fmt, " REVOKE");
				match &stmt.gr {
					Some(v) => write_sql!(f, fmt, " GRANT {}", EscapeKwFreeIdent(v)),
					None => match &stmt.cond {
						Some(v) => write_sql!(f, fmt, " {v}"),
						None => write_sql!(f, fmt, " ALL"),
					},
				};
			}
			Self::Purge(stmt) => {
				write_sql!(f, fmt, "ACCESS {}", EscapeKwFreeIdent(&stmt.ac));
				if let Some(ref v) = stmt.base {
					write_sql!(f, fmt, " ON {v}");
				}
				f.push_str(" PURGE");
				match stmt.kind {
					PurgeKind::Expired => f.push_str(" EXPIRED"),
					PurgeKind::Revoked => f.push_str(" REVOKED"),
					PurgeKind::Both => f.push_str(" EXPIRED, REVOKED"),
				}
				if !stmt.grace.is_zero() {
					write_sql!(f, fmt, " FOR {}", stmt.grace);
				}
			}
		}
	}
}
