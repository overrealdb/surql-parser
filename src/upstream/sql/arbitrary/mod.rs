mod idiom;
mod parts;
mod statements;
mod utils;
use crate::compat::val::Bytes;
use crate::upstream::sql::changefeed::ChangeFeed;
use crate::upstream::sql::statements::SleepStatement;
use arbitrary::{Arbitrary, Result, Unstructured};
pub use idiom::*;
pub use parts::*;
use rust_decimal::Decimal;
use std::time;
use surrealdb_types::Duration;
pub use utils::*;
impl<'a> Arbitrary<'a> for ChangeFeed {
	fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
		Ok(Self {
			expiry: u.arbitrary()?,
			store_diff: bool::arbitrary(u)?,
		})
	}
}
impl<'a> Arbitrary<'a> for SleepStatement {
	fn arbitrary(_u: &mut Unstructured<'a>) -> Result<Self> {
		Ok(Self {
			duration: Duration::from_std(time::Duration::new(0, 0)),
		})
	}
}
impl<'a> Arbitrary<'a> for Bytes {
	fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
		Ok(Bytes(::bytes::Bytes::copy_from_slice(u.arbitrary()?)))
	}
}
pub fn arb_decimal<'a>(u: &mut Unstructured<'a>) -> Result<Decimal> {
	Ok(Decimal::arbitrary(u)?.normalize())
}
