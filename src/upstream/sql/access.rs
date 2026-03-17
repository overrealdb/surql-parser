use crate::compat::types::PublicDuration;
use crate::upstream::sql::{Expr, Literal};
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct AccessDuration {
	pub grant: Expr,
	pub token: Expr,
	pub session: Expr,
}
impl Default for AccessDuration {
	fn default() -> Self {
		Self {
			grant: Expr::Literal(Literal::Duration(
				PublicDuration::from_days(30).expect("30 days should fit in a duration"),
			)),
			token: Expr::Literal(Literal::Duration(
				PublicDuration::from_hours(1).expect("1 hour should fit in a duration"),
			)),
			session: Expr::Literal(Literal::None),
		}
	}
}
