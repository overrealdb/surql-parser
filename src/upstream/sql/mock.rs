use crate::compat::val::range::TypedRange;
use crate::upstream::fmt::EscapeKwFreeIdent;
use std::ops::Bound;
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum Mock {
	Count(String, i64),
	Range(String, TypedRange<i64>),
}
impl ToSql for Mock {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		match self {
			Mock::Count(tb, c) => {
				write_sql!(f, fmt, "|{}:{}|", EscapeKwFreeIdent(tb), c);
			}
			Mock::Range(tb, r) => {
				write_sql!(f, fmt, "|{}:", EscapeKwFreeIdent(tb));
				match r.start {
					Bound::Included(x) => write_sql!(f, fmt, "{x}.."),
					Bound::Excluded(x) => write_sql!(f, fmt, "{x}>.."),
					Bound::Unbounded => f.push_str(".."),
				}
				match r.end {
					Bound::Included(x) => write_sql!(f, fmt, "={x}|"),
					Bound::Excluded(x) => write_sql!(f, fmt, "{x}|"),
					Bound::Unbounded => f.push('|'),
				}
			}
		}
	}
}
