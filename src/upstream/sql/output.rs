use crate::upstream::fmt::CoverStmts;
use crate::upstream::sql::Field;
use crate::upstream::sql::field::{Fields, Selector};
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum Output {
	#[default]
	None,
	Null,
	Diff,
	After,
	Before,
	Fields(Fields),
}
impl ToSql for Output {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		f.push_str("RETURN ");
		match self {
			Self::None => f.push_str("NONE"),
			Self::Null => f.push_str("NULL"),
			Self::Diff => f.push_str("DIFF"),
			Self::After => f.push_str("AFTER"),
			Self::Before => f.push_str("BEFORE"),
			Self::Fields(v) => match v {
				Fields::Select(fields) => {
					let mut iter = fields.iter();
					match iter.next() {
						Some(Field::Single(Selector { expr, alias })) => {
							let has_left_none = expr.has_left_none_null();
							if has_left_none {
								f.push('(');
								expr.fmt_sql(f, fmt);
								f.push(')');
							} else {
								CoverStmts(expr).fmt_sql(f, fmt);
							}
							if let Some(alias) = alias {
								write_sql!(f, fmt, " AS {alias}");
							}
						}
						Some(x) => {
							x.fmt_sql(f, fmt);
						}
						None => {}
					}
					for x in iter {
						write_sql!(f, fmt, ", {x}")
					}
				}
				x => x.fmt_sql(f, fmt),
			},
		}
	}
}
