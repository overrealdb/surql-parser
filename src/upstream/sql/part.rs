use crate::upstream::fmt::{CoverStmts, EscapeKwFreeIdent, Fmt};
use crate::upstream::sql::{Expr, Idiom, Lookup};
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Part {
	All,
	Flatten,
	Last,
	First,
	Field(String),
	Where(Expr),
	Graph(Lookup),
	Value(Expr),
	Start(Expr),
	Method(String, Vec<Expr>),
	Destructure(Vec<DestructurePart>),
	Optional,
	Recurse(Recurse, Option<Idiom>, Option<RecurseInstruction>),
	Doc,
	RepeatRecurse,
}
impl ToSql for Part {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		match self {
			Part::All => f.push_str(".*"),
			Part::Last => f.push_str("[$]"),
			Part::First => f.push_str("[0]"),
			Part::Start(v) => v.fmt_sql(f, fmt),
			Part::Field(v) => write_sql!(f, fmt, ".{}", EscapeKwFreeIdent(v)),
			Part::Flatten => f.push('…'),
			Part::Where(v) => write_sql!(f, fmt, "[WHERE {v}]"),
			Part::Graph(v) => v.fmt_sql(f, fmt),
			Part::Value(v) => write_sql!(f, fmt, "[{v}]"),
			Part::Method(v, a) => {
				write_sql!(
					f,
					fmt,
					".{}({})",
					EscapeKwFreeIdent(v),
					Fmt::comma_separated(a.iter().map(CoverStmts))
				)
			}
			Part::Destructure(v) => {
				f.push_str(".{");
				if !fmt.is_pretty() {
					f.push(' ');
				}
				if !v.is_empty() {
					let fmt = fmt.increment();
					write_sql!(f, fmt, "{}", Fmt::pretty_comma_separated(v));
				}
				if fmt.is_pretty() {
					f.push('}');
				} else {
					f.push_str(" }");
				}
			}
			Part::Optional => f.push_str(".?"),
			Part::Recurse(v, nest, instruction) => {
				write_sql!(f, fmt, ".{{{v}");
				if let Some(instruction) = instruction {
					write_sql!(f, fmt, "+{instruction}");
				}
				f.push('}');
				if let Some(nest) = nest {
					f.push('(');
					for p in nest.0.iter() {
						p.fmt_sql(f, fmt);
					}
					f.push(')');
				}
			}
			Part::Doc => f.push('@'),
			Part::RepeatRecurse => f.push_str(".@"),
		}
	}
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum DestructurePart {
	All(String),
	Field(String),
	Aliased(String, Idiom),
	Destructure(String, Vec<DestructurePart>),
}
impl ToSql for DestructurePart {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		match self {
			DestructurePart::All(fd) => write_sql!(f, fmt, "{}.*", EscapeKwFreeIdent(fd)),
			DestructurePart::Field(fd) => write_sql!(f, fmt, "{}", EscapeKwFreeIdent(fd)),
			DestructurePart::Aliased(fd, v) => {
				write_sql!(f, fmt, "{}: {v}", EscapeKwFreeIdent(fd))
			}
			DestructurePart::Destructure(fd, d) => {
				write_sql!(
					f,
					fmt,
					"{}{}",
					EscapeKwFreeIdent(fd),
					Part::Destructure(d.clone())
				)
			}
		}
	}
}
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum Recurse {
	Fixed(u32),
	Range(Option<u32>, Option<u32>),
}
impl ToSql for Recurse {
	fn fmt_sql(&self, f: &mut String, _fmt: SqlFormat) {
		match self {
			Recurse::Fixed(v) => f.push_str(&v.to_string()),
			Recurse::Range(beg, end) => match (beg, end) {
				(None, None) => f.push_str(".."),
				(Some(beg), None) => {
					f.push_str(&beg.to_string());
					f.push_str("..");
				}
				(None, Some(end)) => {
					f.push_str("..");
					f.push_str(&end.to_string());
				}
				(Some(beg), Some(end)) => {
					f.push_str(&beg.to_string());
					f.push_str("..");
					f.push_str(&end.to_string());
				}
			},
		}
	}
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecurseInstruction {
	Path { inclusive: bool },
	Collect { inclusive: bool },
	Shortest { expects: Expr, inclusive: bool },
}
impl ToSql for RecurseInstruction {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		match self {
			Self::Path { inclusive } => {
				f.push_str("path");
				if *inclusive {
					f.push_str("+inclusive");
				}
			}
			Self::Collect { inclusive } => {
				f.push_str("collect");
				if *inclusive {
					f.push_str("+inclusive");
				}
			}
			Self::Shortest { expects, inclusive } => {
				write_sql!(f, fmt, "shortest={expects}");
				if *inclusive {
					f.push_str("+inclusive");
				}
			}
		}
	}
}
