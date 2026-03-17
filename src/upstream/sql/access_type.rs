use super::Expr;
use crate::compat::err::Error;
use crate::upstream::fmt::CoverStmts;
use crate::upstream::sql::{Algorithm, Literal};
use anyhow::Result;
use rand::Rng;
use rand::distributions::Alphanumeric;
use std::str::FromStr;
use surrealdb_types::{SqlFormat, ToSql, write_sql};
pub fn random_key() -> String {
	rand::thread_rng()
		.sample_iter(&Alphanumeric)
		.take(128)
		.map(char::from)
		.collect::<String>()
}
/// The type of access methods available
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum AccessType {
	Record(RecordAccess),
	Jwt(JwtAccess),
	Bearer(BearerAccess),
}
impl Default for AccessType {
	fn default() -> Self {
		Self::Record(RecordAccess {
			..Default::default()
		})
	}
}
impl ToSql for AccessType {
	fn fmt_sql(&self, f: &mut String, sql_fmt: SqlFormat) {
		match self {
			AccessType::Jwt(ac) => {
				write_sql!(f, sql_fmt, "JWT {}", ac);
			}
			AccessType::Record(ac) => {
				write_sql!(f, sql_fmt, "RECORD");
				if let Some(ref v) = ac.signup {
					write_sql!(f, sql_fmt, " SIGNUP {}", CoverStmts(v));
				}
				if let Some(ref v) = ac.signin {
					write_sql!(f, sql_fmt, " SIGNIN {}", CoverStmts(v));
				}
				if ac.bearer.is_some() {
					write_sql!(f, sql_fmt, " WITH REFRESH")
				}
				write_sql!(f, sql_fmt, " WITH JWT {}", ac.jwt);
			}
			AccessType::Bearer(ac) => {
				write_sql!(f, sql_fmt, "BEARER");
				match ac.subject {
					BearerAccessSubject::User => write_sql!(f, sql_fmt, " FOR USER"),
					BearerAccessSubject::Record => write_sql!(f, sql_fmt, " FOR RECORD"),
				}
			}
		}
	}
}
impl AccessType {
	/// Returns whether or not the access method can issue non-token grants
	/// In this context, token refers exclusively to JWT
	#[allow(dead_code)]
	pub fn can_issue_grants(&self) -> bool {
		match self {
			AccessType::Jwt(_) => false,
			AccessType::Record(ac) => ac.bearer.is_some(),
			AccessType::Bearer(_) => true,
		}
	}
	/// Returns whether or not the access method can issue tokens
	/// In this context, tokens refers exclusively to JWT
	#[allow(dead_code)]
	pub fn can_issue_tokens(&self) -> bool {
		match self {
			AccessType::Jwt(jwt) => jwt.issue.is_some(),
			_ => true,
		}
	}
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JwtAccess {
	pub verify: JwtAccessVerify,
	pub issue: Option<JwtAccessIssue>,
}
impl Default for JwtAccess {
	fn default() -> Self {
		let alg = Algorithm::Hs512;
		let key = random_key();
		Self {
			verify: JwtAccessVerify::Key(JwtAccessVerifyKey {
				alg,
				key: Expr::Literal(Literal::String(key.clone())),
			}),
			issue: Some(JwtAccessIssue {
				alg,
				key: Expr::Literal(Literal::String(key)),
			}),
		}
	}
}
impl ToSql for JwtAccess {
	fn fmt_sql(&self, f: &mut String, sql_fmt: SqlFormat) {
		match &self.verify {
			JwtAccessVerify::Key(v) => {
				write_sql!(f, sql_fmt, "ALGORITHM {} KEY {}", v.alg, CoverStmts(&v.key));
			}
			JwtAccessVerify::Jwks(v) => {
				write_sql!(f, sql_fmt, "URL {}", CoverStmts(&v.url));
			}
		}
		if let Some(iss) = &self.issue {
			write_sql!(f, sql_fmt, " WITH ISSUER KEY {}", CoverStmts(&iss.key));
		}
	}
}
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct JwtAccessIssue {
	pub alg: Algorithm,
	pub key: Expr,
}
impl Default for JwtAccessIssue {
	fn default() -> Self {
		Self {
			alg: Algorithm::Hs512,
			key: Expr::Literal(Literal::String(random_key())),
		}
	}
}
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum JwtAccessVerify {
	Key(JwtAccessVerifyKey),
	Jwks(JwtAccessVerifyJwks),
}
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct JwtAccessVerifyKey {
	pub alg: Algorithm,
	pub key: Expr,
}
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct JwtAccessVerifyJwks {
	pub url: Expr,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct RecordAccess {
	pub signup: Option<Expr>,
	pub signin: Option<Expr>,
	pub jwt: JwtAccess,
	pub bearer: Option<BearerAccess>,
}
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct BearerAccess {
	pub kind: BearerAccessType,
	pub subject: BearerAccessSubject,
	pub jwt: JwtAccess,
}
impl Default for BearerAccess {
	fn default() -> Self {
		Self {
			kind: BearerAccessType::Bearer,
			subject: BearerAccessSubject::User,
			jwt: JwtAccess::default(),
		}
	}
}
#[derive(Debug, Hash, Clone, Eq, PartialEq, PartialOrd)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum BearerAccessType {
	Bearer,
	Refresh,
}
impl FromStr for BearerAccessType {
	type Err = Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_ascii_lowercase().as_str() {
			"bearer" => Ok(Self::Bearer),
			"refresh" => Ok(Self::Refresh),
			_ => Err(Error::AccessGrantBearerInvalid),
		}
	}
}
#[derive(Debug, Hash, Clone, Eq, PartialEq, PartialOrd)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum BearerAccessSubject {
	Record,
	User,
}
