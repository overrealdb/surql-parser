use crate::compat::types::PublicDuration;
use crate::upstream::fmt::{EscapeKwFreeIdent, EscapeObjectKey, Float, Fmt, QuoteStr};
use rust_decimal::Decimal;
use std::collections::{BTreeMap, HashSet};
use std::fmt::Display;
use std::hash;
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum GeometryKind {
	Point,
	Line,
	Polygon,
	MultiPoint,
	MultiLine,
	MultiPolygon,
	Collection,
}
impl ToSql for GeometryKind {
	fn fmt_sql(&self, f: &mut String, _fmt: SqlFormat) {
		match self {
			GeometryKind::Point => f.push_str("point"),
			GeometryKind::Line => f.push_str("line"),
			GeometryKind::Polygon => f.push_str("polygon"),
			GeometryKind::MultiPoint => f.push_str("multipoint"),
			GeometryKind::MultiLine => f.push_str("multiline"),
			GeometryKind::MultiPolygon => f.push_str("multipolygon"),
			GeometryKind::Collection => f.push_str("collection"),
		}
	}
}
impl From<GeometryKind> for crate::compat::types::PublicGeometryKind {
	fn from(v: GeometryKind) -> Self {
		match v {
			GeometryKind::Point => crate::compat::types::PublicGeometryKind::Point,
			GeometryKind::Line => crate::compat::types::PublicGeometryKind::Line,
			GeometryKind::Polygon => crate::compat::types::PublicGeometryKind::Polygon,
			GeometryKind::MultiPoint => crate::compat::types::PublicGeometryKind::MultiPoint,
			GeometryKind::MultiLine => crate::compat::types::PublicGeometryKind::MultiLine,
			GeometryKind::MultiPolygon => crate::compat::types::PublicGeometryKind::MultiPolygon,
			GeometryKind::Collection => crate::compat::types::PublicGeometryKind::Collection,
		}
	}
}
impl From<crate::compat::types::PublicGeometryKind> for GeometryKind {
	fn from(v: crate::compat::types::PublicGeometryKind) -> Self {
		match v {
			crate::compat::types::PublicGeometryKind::Point => GeometryKind::Point,
			crate::compat::types::PublicGeometryKind::Line => GeometryKind::Line,
			crate::compat::types::PublicGeometryKind::Polygon => GeometryKind::Polygon,
			crate::compat::types::PublicGeometryKind::MultiPoint => GeometryKind::MultiPoint,
			crate::compat::types::PublicGeometryKind::MultiLine => GeometryKind::MultiLine,
			crate::compat::types::PublicGeometryKind::MultiPolygon => GeometryKind::MultiPolygon,
			crate::compat::types::PublicGeometryKind::Collection => GeometryKind::Collection,
		}
	}
}
/// The kind, or data type, of a value or field.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum Kind {
	/// The most generic type, can be anything.
	#[default]
	Any,
	/// None type.
	None,
	/// Null type.
	Null,
	/// Boolean type.
	Bool,
	/// Bytes type.
	Bytes,
	/// Datetime type.
	Datetime,
	/// Decimal type.
	Decimal,
	/// Duration type.
	Duration,
	/// 64-bit floating point type.
	Float,
	/// 64-bit signed integer type.
	Int,
	/// Number type, can be either a float, int or decimal.
	/// This is the most generic type for numbers.
	Number,
	/// Object type.
	Object,
	/// String type.
	String,
	/// UUID type.
	Uuid,
	/// Regular expression type.
	Regex,
	/// A table type.
	Table(Vec<String>),
	/// A record type.
	Record(Vec<String>),
	/// A geometry type.
	Geometry(Vec<GeometryKind>),
	/// An either type.
	/// Can be any of the kinds in the vec.
	Either(
		#[cfg_attr(
            feature = "arbitrary",
            arbitrary(with = crate::upstream::sql::arbitrary::either_kind)
        )]
		Vec<Kind>,
	),
	/// A set type.
	Set(Box<Kind>, Option<u64>),
	/// An array type.
	Array(Box<Kind>, Option<u64>),
	/// A function type.
	/// The first option is the argument types, the second is the optional
	/// return type.
	Function(Option<Vec<Kind>>, Option<Box<Kind>>),
	/// A range type.
	Range,
	/// A literal type.
	/// The literal type is used to represent a type that can only be a single
	/// value. For example, `"a"` is a literal type which can only ever be
	/// `"a"`. This can be used in the `Kind::Either` type to represent an
	/// enum.
	Literal(KindLiteral),
	/// A file type.
	/// If the kind was specified without a bucket the vec will be empty.
	/// So `<file>` is just `Kind::File(Vec::new())`
	File(Vec<String>),
}
impl Kind {
	pub fn flatten(self) -> Vec<Kind> {
		match self {
			Kind::Either(x) => x.into_iter().flat_map(|k| k.flatten()).collect(),
			_ => vec![self],
		}
	}
	pub fn either(kinds: Vec<Kind>) -> Kind {
		let mut seen = HashSet::new();
		let mut kinds = kinds
			.into_iter()
			.flat_map(|k| k.flatten())
			.filter(|k| seen.insert(k.clone()))
			.collect::<Vec<_>>();
		match kinds.len() {
			0 => Kind::None,
			1 => kinds.remove(0),
			_ => Kind::Either(kinds),
		}
	}
}
impl From<Kind> for crate::compat::types::PublicKind {
	fn from(v: Kind) -> Self {
		match v {
			Kind::Any => crate::compat::types::PublicKind::Any,
			Kind::None => crate::compat::types::PublicKind::None,
			Kind::Null => crate::compat::types::PublicKind::Null,
			Kind::Bool => crate::compat::types::PublicKind::Bool,
			Kind::Bytes => crate::compat::types::PublicKind::Bytes,
			Kind::Datetime => crate::compat::types::PublicKind::Datetime,
			Kind::Decimal => crate::compat::types::PublicKind::Decimal,
			Kind::Duration => crate::compat::types::PublicKind::Duration,
			Kind::Float => crate::compat::types::PublicKind::Float,
			Kind::Int => crate::compat::types::PublicKind::Int,
			Kind::Number => crate::compat::types::PublicKind::Number,
			Kind::Object => crate::compat::types::PublicKind::Object,
			Kind::String => crate::compat::types::PublicKind::String,
			Kind::Uuid => crate::compat::types::PublicKind::Uuid,
			Kind::Regex => crate::compat::types::PublicKind::Regex,
			Kind::Table(k) => {
				crate::compat::types::PublicKind::Table(k.into_iter().map(Into::into).collect())
			}
			Kind::Record(k) => {
				crate::compat::types::PublicKind::Record(k.into_iter().map(Into::into).collect())
			}
			Kind::Geometry(k) => {
				crate::compat::types::PublicKind::Geometry(k.into_iter().map(Into::into).collect())
			}
			Kind::Either(k) => {
				crate::compat::types::PublicKind::Either(k.into_iter().map(Into::into).collect())
			}
			Kind::Set(k, l) => crate::compat::types::PublicKind::Set(Box::new((*k).into()), l),
			Kind::Array(k, l) => crate::compat::types::PublicKind::Array(Box::new((*k).into()), l),
			Kind::Function(args, ret) => crate::compat::types::PublicKind::Function(
				args.map(|args| args.into_iter().map(Into::into).collect()),
				ret.map(|ret| Box::new((*ret).into())),
			),
			Kind::Range => crate::compat::types::PublicKind::Range,
			Kind::Literal(l) => crate::compat::types::PublicKind::Literal(l.into()),
			Kind::File(k) => crate::compat::types::PublicKind::File(k),
		}
	}
}
impl From<crate::compat::types::PublicKind> for Kind {
	fn from(v: crate::compat::types::PublicKind) -> Self {
		match v {
			crate::compat::types::PublicKind::None => Kind::None,
			crate::compat::types::PublicKind::Null => Kind::Null,
			crate::compat::types::PublicKind::Any => Kind::Any,
			crate::compat::types::PublicKind::Bool => Kind::Bool,
			crate::compat::types::PublicKind::Bytes => Kind::Bytes,
			crate::compat::types::PublicKind::Datetime => Kind::Datetime,
			crate::compat::types::PublicKind::Decimal => Kind::Decimal,
			crate::compat::types::PublicKind::Duration => Kind::Duration,
			crate::compat::types::PublicKind::Float => Kind::Float,
			crate::compat::types::PublicKind::Int => Kind::Int,
			crate::compat::types::PublicKind::Number => Kind::Number,
			crate::compat::types::PublicKind::Object => Kind::Object,
			crate::compat::types::PublicKind::String => Kind::String,
			crate::compat::types::PublicKind::Uuid => Kind::Uuid,
			crate::compat::types::PublicKind::Regex => Kind::Regex,
			crate::compat::types::PublicKind::Table(k) => {
				Kind::Table(k.into_iter().map(Into::into).collect())
			}
			crate::compat::types::PublicKind::Record(k) => {
				Kind::Record(k.into_iter().map(Into::into).collect())
			}
			crate::compat::types::PublicKind::Geometry(k) => {
				Kind::Geometry(k.into_iter().map(Into::into).collect())
			}
			crate::compat::types::PublicKind::Either(k) => {
				Kind::Either(k.into_iter().map(Into::into).collect())
			}
			crate::compat::types::PublicKind::Set(k, l) => Kind::Set(Box::new((*k).into()), l),
			crate::compat::types::PublicKind::Array(k, l) => Kind::Array(Box::new((*k).into()), l),
			crate::compat::types::PublicKind::Function(args, ret) => Kind::Function(
				args.map(|args| args.into_iter().map(Into::into).collect()),
				ret.map(|ret| Box::new((*ret).into())),
			),
			crate::compat::types::PublicKind::Range => Kind::Range,
			crate::compat::types::PublicKind::Literal(l) => Kind::Literal(l.into()),
			crate::compat::types::PublicKind::File(k) => Kind::File(k),
		}
	}
}
impl ToSql for Kind {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		match self {
			Kind::Any => f.push_str("any"),
			Kind::None => f.push_str("none"),
			Kind::Null => f.push_str("null"),
			Kind::Bool => f.push_str("bool"),
			Kind::Bytes => f.push_str("bytes"),
			Kind::Datetime => f.push_str("datetime"),
			Kind::Decimal => f.push_str("decimal"),
			Kind::Duration => f.push_str("duration"),
			Kind::Float => f.push_str("float"),
			Kind::Int => f.push_str("int"),
			Kind::Number => f.push_str("number"),
			Kind::Object => f.push_str("object"),
			Kind::String => f.push_str("string"),
			Kind::Uuid => f.push_str("uuid"),
			Kind::Regex => f.push_str("regex"),
			Kind::Function(_, _) => f.push_str("function"),
			Kind::Table(k) => {
				if k.is_empty() {
					f.push_str("table");
				} else {
					write_sql!(
						f,
						fmt,
						"table<{}>",
						Fmt::verbar_separated(k.iter().map(|x| EscapeKwFreeIdent(x)))
					);
				}
			}
			Kind::Record(k) => {
				if k.is_empty() {
					f.push_str("record");
				} else {
					write_sql!(
						f,
						fmt,
						"record<{}>",
						Fmt::verbar_separated(k.iter().map(|x| EscapeKwFreeIdent(x)))
					);
				}
			}
			Kind::Geometry(k) => {
				if k.is_empty() {
					f.push_str("geometry");
				} else {
					write_sql!(f, fmt, "geometry<{}>", Fmt::verbar_separated(k));
				}
			}
			Kind::Set(k, l) => match (k, l) {
				(k, None) if matches!(**k, Kind::Any) => f.push_str("set"),
				(k, None) => write_sql!(f, fmt, "set<{k}>"),
				(k, Some(l)) => write_sql!(f, fmt, "set<{k}, {l}>"),
			},
			Kind::Array(k, l) => match (k, l) {
				(k, None) if matches!(**k, Kind::Any) => f.push_str("array"),
				(k, None) => write_sql!(f, fmt, "array<{k}>"),
				(k, Some(l)) => write_sql!(f, fmt, "array<{k}, {l}>"),
			},
			Kind::Either(k) => write_sql!(f, fmt, "{}", Fmt::verbar_separated(k)),
			Kind::Range => f.push_str("range"),
			Kind::Literal(l) => l.fmt_sql(f, fmt),
			Kind::File(k) => {
				if k.is_empty() {
					f.push_str("file");
				} else {
					write_sql!(
						f,
						fmt,
						"file<{}>",
						Fmt::verbar_separated(k.iter().map(|x| EscapeKwFreeIdent(x)))
					);
				}
			}
		}
	}
}
#[derive(Clone, Debug)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum KindLiteral {
	String(String),
	Integer(i64),
	Float(f64),
	Decimal(Decimal),
	Duration(PublicDuration),
	Array(Vec<Kind>),
	Object(BTreeMap<String, Kind>),
	Bool(bool),
}
impl hash::Hash for KindLiteral {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		match self {
			Self::String(v) => v.hash(state),
			Self::Integer(v) => v.hash(state),
			Self::Float(v) => v.to_bits().hash(state),
			Self::Decimal(v) => v.hash(state),
			Self::Duration(v) => v.hash(state),
			Self::Array(v) => v.hash(state),
			Self::Object(v) => v.hash(state),
			Self::Bool(v) => v.hash(state),
		}
	}
}
impl PartialEq for KindLiteral {
	fn eq(&self, other: &Self) -> bool {
		match self {
			KindLiteral::String(a) => {
				if let KindLiteral::String(b) = other {
					a == b
				} else {
					false
				}
			}
			KindLiteral::Integer(a) => {
				if let KindLiteral::Integer(b) = other {
					a == b
				} else {
					false
				}
			}
			KindLiteral::Float(a) => {
				if let KindLiteral::Float(b) = other {
					a.to_bits() == b.to_bits()
				} else {
					false
				}
			}
			KindLiteral::Decimal(a) => {
				if let KindLiteral::Decimal(b) = other {
					a == b
				} else {
					false
				}
			}
			KindLiteral::Duration(a) => {
				if let KindLiteral::Duration(b) = other {
					a == b
				} else {
					false
				}
			}
			KindLiteral::Array(a) => {
				if let KindLiteral::Array(b) = other {
					a == b
				} else {
					false
				}
			}
			KindLiteral::Object(a) => {
				if let KindLiteral::Object(b) = other {
					a == b
				} else {
					false
				}
			}
			KindLiteral::Bool(a) => {
				if let KindLiteral::Bool(b) = other {
					a == b
				} else {
					false
				}
			}
		}
	}
}
impl Eq for KindLiteral {}
impl ToSql for KindLiteral {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		match self {
			KindLiteral::String(s) => write_sql!(f, fmt, "{}", QuoteStr(s)),
			KindLiteral::Integer(n) => write_sql!(f, fmt, "{}", n),
			KindLiteral::Float(n) => write_sql!(f, fmt, " {}", Float(*n)),
			KindLiteral::Decimal(n) => write_sql!(f, fmt, " {}", n),
			KindLiteral::Duration(d) => write_sql!(f, fmt, "{}", d),
			KindLiteral::Bool(b) => write_sql!(f, fmt, "{}", b),
			KindLiteral::Array(a) => {
				f.push('[');
				if !a.is_empty() {
					let fmt = fmt.increment();
					write_sql!(f, fmt, "{}", Fmt::pretty_comma_separated(a.as_slice()));
				}
				f.push(']');
			}
			KindLiteral::Object(o) => {
				if fmt.is_pretty() {
					f.push('{');
				} else {
					f.push_str("{ ");
				}
				if !o.is_empty() {
					let fmt = fmt.increment();
					write_sql!(
						f,
						fmt,
						"{}",
						Fmt::pretty_comma_separated(o.iter().map(|args| Fmt::new(
							args,
							|(k, v), f, fmt| {
								write_sql!(f, fmt, "{}: {}", EscapeObjectKey(k), v)
							}
						)),)
					);
				}
				if fmt.is_pretty() {
					f.push('}');
				} else {
					f.push_str(" }");
				}
			}
		}
	}
}
impl Display for Kind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.to_sql())
	}
}
impl From<KindLiteral> for crate::compat::types::PublicKindLiteral {
	fn from(v: KindLiteral) -> Self {
		match v {
			KindLiteral::Bool(b) => crate::compat::types::PublicKindLiteral::Bool(b),
			KindLiteral::Integer(i) => crate::compat::types::PublicKindLiteral::Integer(i),
			KindLiteral::Float(f) => crate::compat::types::PublicKindLiteral::Float(f),
			KindLiteral::Decimal(d) => crate::compat::types::PublicKindLiteral::Decimal(d),
			KindLiteral::String(s) => crate::compat::types::PublicKindLiteral::String(s),
			KindLiteral::Duration(d) => crate::compat::types::PublicKindLiteral::Duration(d),
			KindLiteral::Array(a) => crate::compat::types::PublicKindLiteral::Array(
				a.into_iter().map(Into::into).collect(),
			),
			KindLiteral::Object(o) => crate::compat::types::PublicKindLiteral::Object(
				o.into_iter().map(|(k, v)| (k, v.into())).collect(),
			),
		}
	}
}
impl From<crate::compat::types::PublicKindLiteral> for KindLiteral {
	fn from(v: crate::compat::types::PublicKindLiteral) -> Self {
		match v {
			crate::compat::types::PublicKindLiteral::Bool(b) => Self::Bool(b),
			crate::compat::types::PublicKindLiteral::Integer(i) => Self::Integer(i),
			crate::compat::types::PublicKindLiteral::Float(f) => Self::Float(f),
			crate::compat::types::PublicKindLiteral::Decimal(d) => Self::Decimal(d),
			crate::compat::types::PublicKindLiteral::String(s) => Self::String(s),
			crate::compat::types::PublicKindLiteral::Duration(d) => Self::Duration(d),
			crate::compat::types::PublicKindLiteral::Array(a) => {
				Self::Array(a.into_iter().map(Into::into).collect())
			}
			crate::compat::types::PublicKindLiteral::Object(o) => {
				Self::Object(o.into_iter().map(|(k, v)| (k, v.into())).collect())
			}
		}
	}
}
