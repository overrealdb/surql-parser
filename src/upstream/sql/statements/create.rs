use crate::upstream::fmt::{CoverStmts, Fmt};
use crate::upstream::sql::{Data, Expr, Literal, Output};
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct CreateStatement {
	pub only: bool,
	#[cfg_attr(
        feature = "arbitrary",
        arbitrary(with = crate::upstream::sql::arbitrary::atleast_one)
    )]
	pub what: Vec<Expr>,
	pub data: Option<Data>,
	pub output: Option<Output>,
	pub timeout: Expr,
}
impl Default for CreateStatement {
	fn default() -> Self {
		Self {
			only: Default::default(),
			what: Default::default(),
			data: Default::default(),
			output: Default::default(),
			timeout: Expr::Literal(Literal::None),
		}
	}
}
impl ToSql for CreateStatement {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		write_sql!(f, fmt, "CREATE");
		if self.only {
			write_sql!(f, fmt, " ONLY");
		}
		write_sql!(
			f,
			fmt,
			" {}",
			Fmt::comma_separated(self.what.iter().map(CoverStmts))
		);
		if let Some(ref v) = self.data {
			write_sql!(f, fmt, " {v}");
		}
		if let Some(ref v) = self.output {
			write_sql!(f, fmt, " {v}");
		}
		if !matches!(self.timeout, Expr::Literal(Literal::None)) {
			write_sql!(f, fmt, " TIMEOUT {}", CoverStmts(&self.timeout));
		}
	}
}
