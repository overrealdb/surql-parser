use crate::upstream::sql::{BinaryOperator, Expr, Literal};
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Block(pub Vec<Expr>);
impl ToSql for Block {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		match self.0.len() {
			0 => f.push_str("{;}"),
			1 => {
				let v = &self.0[0];
				if fmt.is_pretty() {
					f.push('{');
					f.push('\n');
					f.push('\n');
					let fmt = fmt.increment();
					fmt.write_indent(f);
					if let Expr::Literal(Literal::RecordId(_)) = v {
						write_sql!(f, fmt, "({v})");
					} else if let Expr::Binary {
						left,
						op: BinaryOperator::Equal,
						..
					} = v && let Expr::Param(_) = **left
					{
						write_sql!(f, fmt, "({v})");
					} else {
						v.fmt_sql(f, fmt);
					}
					f.push('\n');
					if let SqlFormat::Indented(level) = fmt
						&& level > 0
					{
						for _ in 0..(level - 1) {
							f.push('\t');
						}
					}
					f.push('}')
				} else {
					f.push_str("{ ");
					if let Expr::Literal(Literal::RecordId(_)) = v {
						write_sql!(f, fmt, "({v})");
					} else if let Expr::Binary {
						left,
						op: BinaryOperator::Equal,
						..
					} = v && let Expr::Param(_) = **left
					{
						write_sql!(f, fmt, "({v})");
					} else {
						v.fmt_sql(f, fmt);
					}
					f.push_str(" }");
				}
			}
			_ => {
				if fmt.is_pretty() {
					f.push('{');
					f.push('\n');
					f.push('\n');
					let fmt = fmt.increment();
					for (i, v) in self.0.iter().enumerate() {
						if i > 0 {
							f.push('\n');
							f.push('\n');
						}
						fmt.write_indent(f);
						if i == 0
							&& let Expr::Literal(Literal::RecordId(_)) = v
						{
							write_sql!(f, fmt, "({v})");
						} else if let Expr::Binary {
							left,
							op: BinaryOperator::Equal,
							..
						} = v && let Expr::Param(_) = **left
						{
							write_sql!(f, fmt, "({v})");
						} else {
							v.fmt_sql(f, fmt);
						}
						f.push(';');
					}
					f.push('\n');
					if let SqlFormat::Indented(level) = fmt
						&& level > 0
					{
						for _ in 0..(level - 1) {
							f.push('\t');
						}
					}
					f.push('}')
				} else {
					f.push_str("{ ");
					for (i, v) in self.0.iter().enumerate() {
						if i > 0 {
							f.push(' ');
						}
						if i == 0
							&& let Expr::Literal(Literal::RecordId(_)) = v
						{
							write_sql!(f, fmt, "({v})");
						} else if let Expr::Binary {
							left,
							op: BinaryOperator::Equal,
							..
						} = v && let Expr::Param(_) = **left
						{
							write_sql!(f, fmt, "({v})");
						} else {
							v.fmt_sql(f, fmt);
						}
						f.push(';');
					}
					f.push_str(" }")
				}
			}
		}
	}
}
