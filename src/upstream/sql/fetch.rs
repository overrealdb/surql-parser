use crate::upstream::fmt::Fmt;
use crate::upstream::sql::Expr;
use std::ops::Deref;
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Fetchs(
	#[cfg_attr(
        feature = "arbitrary",
        arbitrary(with = crate::upstream::sql::arbitrary::atleast_one)
    )]
	pub Vec<Fetch>,
);
impl Deref for Fetchs {
	type Target = Vec<Fetch>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl ToSql for Fetchs {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		write_sql!(f, fmt, "FETCH {}", Fmt::comma_separated(&self.0))
	}
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fetch(pub Expr);
impl ToSql for Fetch {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		self.0.fmt_sql(f, fmt);
	}
}
