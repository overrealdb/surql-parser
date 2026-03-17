use crate::upstream::fmt::{CoverStmts, Fmt, fmt_separated_by};
use crate::upstream::sql::Expr;
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct IfelseStatement {
	/// The first if condition followed by a body, followed by any number of
	/// else if's
	#[cfg_attr(
        feature = "arbitrary",
        arbitrary(with = crate::upstream::sql::arbitrary::atleast_one)
    )]
	pub exprs: Vec<(Expr, Expr)>,
	/// the final else body, if there is one
	pub close: Option<Expr>,
}
impl IfelseStatement {
	/// Check if the statement is bracketed
	pub fn bracketed(&self) -> bool {
		self.exprs.iter().all(|(_, v)| matches!(v, Expr::Block(_)))
			&& self
				.close
				.as_ref()
				.map(|v| matches!(v, Expr::Block(_)))
				.unwrap_or(true)
	}
}
impl ToSql for IfelseStatement {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		if self.bracketed() {
			let is_simple_block = |expr: &Expr| -> bool {
				if let Expr::Block(block) = expr {
					block.0.iter().all(|stmt| {
						matches!(stmt, Expr::Literal(_) | Expr::Param(_) | Expr::Idiom(_))
					})
				} else {
					false
				}
			};
			let has_complex_multi = self.exprs.iter().any(|(_, expr)| {
				matches!(
					expr, Expr::Block(block) if block.0.len() > 1 && !
					is_simple_block(expr)
				)
			}) || self
				.close
				.as_ref()
				.map(|expr| {
					matches!(
						expr, Expr::Block(block) if block.0.len() > 1 && !
						is_simple_block(expr)
					)
				})
				.unwrap_or(false);
			let fmt_block = |f: &mut String, fmt: SqlFormat, expr: &Expr, use_separated: bool| {
				if let Expr::Block(block) = expr {
					match block.0.len() {
						0 => f.push_str("{;}"),
						1 if !use_separated => {
							f.push_str("{ ");
							block.0[0].fmt_sql(f, SqlFormat::SingleLine);
							f.push_str(" }");
						}
						1 => {
							f.push('{');
							f.push(' ');
							block.0[0].fmt_sql(f, SqlFormat::SingleLine);
							f.push(' ');
							f.push('}');
						}
						_ => {
							let needs_indent = is_simple_block(expr);
							if fmt.is_pretty() && !needs_indent {
								f.push_str("{\n\n");
								let inner_fmt = fmt.increment();
								for (i, stmt) in block.0.iter().enumerate() {
									if i > 0 {
										f.push('\n');
										f.push('\n');
									}
									inner_fmt.write_indent(f);
									stmt.fmt_sql(f, SqlFormat::SingleLine);
									f.push(';');
								}
								f.push('\n');
								fmt.write_indent(f);
								f.push('\n');
								f.push('}');
							} else if fmt.is_pretty() {
								f.push_str("{\n\n");
								for (i, stmt) in block.0.iter().enumerate() {
									if i > 0 {
										f.push('\n');
									}
									f.push('\t');
									stmt.fmt_sql(f, SqlFormat::SingleLine);
									f.push(';');
								}
								f.push_str("\n}");
							} else {
								f.push_str("{\n");
								for (i, stmt) in block.0.iter().enumerate() {
									if i > 0 {
										f.push('\n');
									}
									if needs_indent {
										f.push('\t');
									}
									stmt.fmt_sql(f, SqlFormat::SingleLine);
									f.push(';');
								}
								f.push_str("\n}");
							}
						}
					}
				} else {
					expr.fmt_sql(f, fmt);
				}
			};
			let is_nested = matches!(fmt, SqlFormat::Indented(level) if level > 0);
			let use_separated = fmt.is_pretty() && (has_complex_multi || is_nested);
			write_sql!(
				f,
				fmt,
				"{}",
				&Fmt::new(
					self.exprs.iter().map(|args| {
						Fmt::new(args, |(cond, then), f, fmt| {
							if use_separated {
								write_sql!(f, fmt, "IF {}", CoverStmts(cond));
								f.push('\n');
								if is_nested {
									fmt.write_indent(f);
									fmt_block(f, fmt, then, true);
								} else {
									let fmt = fmt.increment();
									fmt.write_indent(f);
									fmt_block(f, fmt, then, true);
								}
							} else {
								write_sql!(f, fmt, "IF {} ", CoverStmts(cond));
								fmt_block(f, fmt, then, false);
							}
						})
					}),
					if use_separated {
						fmt_separated_by("\nELSE ")
					} else {
						fmt_separated_by(" ELSE ")
					},
				),
			);
			if let Some(ref v) = self.close {
				if use_separated {
					f.push('\n');
					write_sql!(f, fmt, "ELSE");
					f.push('\n');
					if is_nested {
						fmt.write_indent(f);
						fmt_block(f, fmt, v, true);
					} else {
						let fmt = fmt.increment();
						fmt.write_indent(f);
						fmt_block(f, fmt, v, true);
					}
				} else {
					write_sql!(f, fmt, " ELSE ");
					fmt_block(f, fmt, v, false);
				}
			}
		} else {
			write_sql!(
				f,
				fmt,
				"{}",
				&Fmt::new(
					self.exprs.iter().map(|args| {
						Fmt::new(args, |(cond, then), f, fmt| {
							if fmt.is_pretty() {
								write_sql!(f, fmt, "IF {} THEN", CoverStmts(cond));
								f.push('\n');
								let fmt = fmt.increment();
								fmt.write_indent(f);
								if let Expr::IfElse(then) = then
									&& then.bracketed()
								{
									write_sql!(f, fmt, "({then})");
								} else {
									write_sql!(f, fmt, "{then}");
								}
							} else {
								write_sql!(f, fmt, "IF {} THEN ", CoverStmts(cond));
								if let Expr::IfElse(then) = then
									&& then.bracketed()
								{
									write_sql!(f, fmt, "({then})");
								} else {
									write_sql!(f, fmt, "{then}");
								}
							}
						})
					}),
					if fmt.is_pretty() {
						fmt_separated_by("\nELSE ")
					} else {
						fmt_separated_by(" ELSE ")
					},
				),
			);
			if let Some(ref v) = self.close {
				if fmt.is_pretty() {
					f.push('\n');
					write_sql!(f, fmt, "ELSE");
					f.push('\n');
					let fmt = fmt.increment();
					fmt.write_indent(f);
					write_sql!(f, fmt, "{}", CoverStmts(v));
				} else {
					write_sql!(f, fmt, " ELSE {}", CoverStmts(v));
				}
			}
			if fmt.is_pretty() {
				write_sql!(f, fmt, "END");
			} else {
				write_sql!(f, fmt, " END");
			}
		}
	}
}
