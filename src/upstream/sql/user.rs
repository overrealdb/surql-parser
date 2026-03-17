use crate::upstream::sql::Expr;
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct UserDuration {
	pub token: Expr,
	pub session: Expr,
}
