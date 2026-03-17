//! SurrealQL formatting utilities.
mod escape;
use crate::compat::fmt::fmt_non_finite_f64;
use crate::upstream::sql;
pub use escape::{
	EscapeIdent, EscapeKwFreeIdent, EscapeKwIdent, EscapeObjectKey, EscapeRidKey, QuoteStr,
};
use std::cell::Cell;
use std::fmt::Display;
use surrealdb_types::{SqlFormat, ToSql};
/// Implements ToSql by calling formatter on contents.
pub struct Fmt<T, F> {
	contents: Cell<Option<T>>,
	formatter: F,
}
impl<T, F: Fn(T, &mut String, SqlFormat)> Fmt<T, F> {
	pub fn new(t: T, formatter: F) -> Self {
		Self {
			contents: Cell::new(Some(t)),
			formatter,
		}
	}
}
impl<T, F: Fn(T, &mut String, SqlFormat)> ToSql for Fmt<T, F> {
	/// fmt is single-use only.
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		let contents = self
			.contents
			.replace(None)
			.expect("only call Fmt::fmt once");
		(self.formatter)(contents, f, fmt)
	}
}
impl<I: IntoIterator<Item = T>, T: ToSql> Fmt<I, fn(I, &mut String, SqlFormat)> {
	/// Formats values with a comma and a space separating them.
	pub fn comma_separated(into_iter: I) -> Self {
		Self::new(into_iter, fmt_comma_separated)
	}
	/// Formats values with a verbar and a space separating them.
	pub fn verbar_separated(into_iter: I) -> Self {
		Self::new(into_iter, fmt_verbar_separated)
	}
	/// Formats values with a comma and a space separating them or, if pretty
	/// printing is in effect, a comma, a newline, and indentation.
	pub fn pretty_comma_separated(into_iter: I) -> Self {
		Self::new(into_iter, fmt_pretty_comma_separated)
	}
	/// Formats values with a new line separating them.
	pub fn one_line_separated(into_iter: I) -> Self {
		Self::new(into_iter, fmt_one_line_separated)
	}
}
fn fmt_comma_separated<T: ToSql, I: IntoIterator<Item = T>>(
	into_iter: I,
	f: &mut String,
	fmt: SqlFormat,
) {
	for (i, v) in into_iter.into_iter().enumerate() {
		if i > 0 {
			f.push_str(", ");
		}
		v.fmt_sql(f, fmt);
	}
}
fn fmt_verbar_separated<T: ToSql, I: IntoIterator<Item = T>>(
	into_iter: I,
	f: &mut String,
	fmt: SqlFormat,
) {
	for (i, v) in into_iter.into_iter().enumerate() {
		if i > 0 {
			f.push_str(" | ");
		}
		v.fmt_sql(f, fmt);
	}
}
fn fmt_pretty_comma_separated<T: ToSql, I: IntoIterator<Item = T>>(
	into_iter: I,
	f: &mut String,
	fmt: SqlFormat,
) {
	for (i, v) in into_iter.into_iter().enumerate() {
		if i > 0 {
			if fmt.is_pretty() {
				f.push_str(",\n");
			} else {
				f.push_str(", ");
			}
		}
		v.fmt_sql(f, fmt);
	}
}
fn fmt_one_line_separated<T: ToSql, I: IntoIterator<Item = T>>(
	into_iter: I,
	f: &mut String,
	fmt: SqlFormat,
) {
	for (i, v) in into_iter.into_iter().enumerate() {
		if i > 0 {
			f.push('\n');
		}
		v.fmt_sql(f, fmt);
	}
}
/// Creates a formatting function that joins iterators with an arbitrary
/// separator.
pub fn fmt_separated_by<T: ToSql, I: IntoIterator<Item = T>>(
	separator: impl Display,
) -> impl Fn(I, &mut String, SqlFormat) {
	move |into_iter: I, f: &mut String, fmt: SqlFormat| {
		let separator = separator.to_string();
		for (i, v) in into_iter.into_iter().enumerate() {
			if i > 0 {
				f.push_str(&separator);
			}
			v.fmt_sql(f, fmt);
		}
	}
}
pub struct CoverStmts<'a>(pub &'a sql::Expr);
impl ToSql for CoverStmts<'_> {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		match self.0 {
			sql::Expr::Literal(_)
			| sql::Expr::Param(_)
			| sql::Expr::Idiom(_)
			| sql::Expr::Table(_)
			| sql::Expr::Mock(_)
			| sql::Expr::Block(_)
			| sql::Expr::Constant(_)
			| sql::Expr::Prefix { .. }
			| sql::Expr::Postfix { .. }
			| sql::Expr::Binary { .. }
			| sql::Expr::FunctionCall(_)
			| sql::Expr::Closure(_)
			| sql::Expr::Break
			| sql::Expr::Continue
			| sql::Expr::Throw(_) => self.0.fmt_sql(f, fmt),
			sql::Expr::Return(x) => {
				if x.fetch.is_some() {
					f.push('(');
					self.0.fmt_sql(f, fmt);
					f.push(')')
				} else {
					self.0.fmt_sql(f, fmt);
				}
			}
			sql::Expr::IfElse(_)
			| sql::Expr::Select(_)
			| sql::Expr::Create(_)
			| sql::Expr::Update(_)
			| sql::Expr::Upsert(_)
			| sql::Expr::Delete(_)
			| sql::Expr::Relate(_)
			| sql::Expr::Insert(_)
			| sql::Expr::Define(_)
			| sql::Expr::Remove(_)
			| sql::Expr::Rebuild(_)
			| sql::Expr::Alter(_)
			| sql::Expr::Info(_)
			| sql::Expr::Foreach(_)
			| sql::Expr::Let(_)
			| sql::Expr::Sleep(_)
			| sql::Expr::Explain { .. } => {
				f.push('(');
				self.0.fmt_sql(f, fmt);
				f.push(')')
			}
		}
	}
}
pub struct Float(pub f64);
impl ToSql for Float {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		match fmt_non_finite_f64(self.0) {
			Some(special) => f.push_str(special),
			None => {
				self.0.fmt_sql(f, fmt);
				f.push('f');
			}
		}
	}
}
