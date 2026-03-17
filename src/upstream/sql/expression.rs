use crate::compat::types::{PublicFile, PublicNumber, PublicRecordId, PublicValue};
use crate::upstream::fmt::{CoverStmts, EscapeIdent};
use crate::upstream::sql::ast::ExplainFormat;
use crate::upstream::sql::literal::ObjectEntry;
use crate::upstream::sql::lookup::LookupKind;
use crate::upstream::sql::operator::BindingPower;
use crate::upstream::sql::statements::{
	AlterStatement, CreateStatement, DefineStatement, DeleteStatement, ForeachStatement,
	IfelseStatement, InfoStatement, InsertStatement, OutputStatement, RebuildStatement,
	RelateStatement, RemoveStatement, SelectStatement, SetStatement, SleepStatement,
	UpdateStatement, UpsertStatement,
};
use crate::upstream::sql::{
	BinaryOperator, Block, Closure, Constant, Dir, FunctionCall, Idiom, Literal, Mock, Param, Part,
	PostfixOperator, PrefixOperator, RecordIdKeyLit, RecordIdLit,
};
use std::ops::Bound;
use surrealdb_types::{SqlFormat, ToSql, write_sql};
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum Expr {
	Literal(Literal),
	Param(Param),
	Idiom(Idiom),
	Table(String),
	Mock(Mock),
	Block(Box<Block>),
	Constant(Constant),
	Prefix {
		op: PrefixOperator,
		expr: Box<Expr>,
	},
	Postfix {
		expr: Box<Expr>,
		op: PostfixOperator,
	},
	Binary {
		left: Box<Expr>,
		op: BinaryOperator,
		right: Box<Expr>,
	},
	FunctionCall(Box<FunctionCall>),
	Closure(Box<Closure>),
	Break,
	Continue,
	Throw(Box<Expr>),
	Return(Box<OutputStatement>),
	IfElse(Box<IfelseStatement>),
	Select(Box<SelectStatement>),
	Create(Box<CreateStatement>),
	Update(Box<UpdateStatement>),
	Delete(Box<DeleteStatement>),
	Relate(Box<RelateStatement>),
	Insert(Box<InsertStatement>),
	Define(Box<DefineStatement>),
	Remove(Box<RemoveStatement>),
	Rebuild(Box<RebuildStatement>),
	Upsert(Box<UpsertStatement>),
	Alter(Box<AlterStatement>),
	Info(Box<InfoStatement>),
	Foreach(Box<ForeachStatement>),
	Let(Box<SetStatement>),
	Sleep(Box<SleepStatement>),
	Explain {
		format: ExplainFormat,
		analyze: bool,
		statement: Box<Expr>,
	},
}
impl Expr {
	pub fn to_idiom(&self) -> Idiom {
		match self {
			Expr::Idiom(i) => i.simplify(),
			Expr::Param(i) => Idiom::field(i.clone().to_string()),
			Expr::FunctionCall(x) => x.receiver.to_idiom(),
			Expr::Literal(l) => match l {
				Literal::String(s) => Idiom::field(s.clone()),
				Literal::Datetime(d) => Idiom::field(d.to_string()),
				x => Idiom::field(x.to_sql()),
			},
			x => Idiom::field(x.to_sql()),
		}
	}
	pub fn from_public_value(value: PublicValue) -> Self {
		match value {
			PublicValue::None => Expr::Literal(Literal::None),
			PublicValue::Null => Expr::Literal(Literal::Null),
			PublicValue::Bool(x) => Expr::Literal(Literal::Bool(x)),
			PublicValue::Number(PublicNumber::Float(x)) => Expr::Literal(Literal::Float(x)),
			PublicValue::Number(PublicNumber::Int(x)) => Expr::Literal(Literal::Integer(x)),
			PublicValue::Number(PublicNumber::Decimal(x)) => Expr::Literal(Literal::Decimal(x)),
			PublicValue::String(x) => Expr::Literal(Literal::String(x)),
			PublicValue::Bytes(x) => Expr::Literal(Literal::Bytes(x)),
			PublicValue::Regex(x) => Expr::Literal(Literal::Regex(x)),
			PublicValue::Table(x) => Expr::Table(x.to_string()),
			PublicValue::RecordId(PublicRecordId { table, key }) => {
				Expr::Literal(Literal::RecordId(RecordIdLit {
					table: table.to_string(),
					key: RecordIdKeyLit::from_record_id_key(key),
				}))
			}
			PublicValue::Array(x) => Expr::Literal(Literal::Array(
				x.into_iter().map(Expr::from_public_value).collect(),
			)),
			PublicValue::Set(x) => Expr::Literal(Literal::Array(
				x.into_iter().map(Expr::from_public_value).collect(),
			)),
			PublicValue::Object(x) => Expr::Literal(Literal::Object(
				x.into_iter()
					.map(|(k, v)| ObjectEntry {
						key: k,
						value: Expr::from_public_value(v),
					})
					.collect(),
			)),
			PublicValue::Duration(x) => Expr::Literal(Literal::Duration(x)),
			PublicValue::Datetime(x) => Expr::Literal(Literal::Datetime(x)),
			PublicValue::Uuid(x) => Expr::Literal(Literal::Uuid(x)),
			PublicValue::Geometry(x) => Expr::Literal(Literal::Geometry(x)),
			PublicValue::File(x) => Expr::Literal(Literal::File(PublicFile::new(x.bucket, x.key))),
			PublicValue::Range(x) => convert_public_range_to_literal(*x),
		}
	}
	/// Returns if this expression needs to be parenthesized when inside another expression.
	pub fn needs_parentheses(&self) -> bool {
		match self {
			Expr::Literal(Literal::UnboundedRange | Literal::RecordId(_))
			| Expr::Closure(_)
			| Expr::Break
			| Expr::Continue
			| Expr::Throw(_)
			| Expr::Return(_)
			| Expr::IfElse(_)
			| Expr::Select(_)
			| Expr::Create(_)
			| Expr::Update(_)
			| Expr::Delete(_)
			| Expr::Relate(_)
			| Expr::Insert(_)
			| Expr::Define(_)
			| Expr::Remove(_)
			| Expr::Rebuild(_)
			| Expr::Upsert(_)
			| Expr::Alter(_)
			| Expr::Info(_)
			| Expr::Foreach(_)
			| Expr::Let(_)
			| Expr::Sleep(_)
			| Expr::Explain { .. } => true,
			Expr::Postfix { op, .. } => {
				matches!(
					op,
					PostfixOperator::Range
						| PostfixOperator::RangeSkip
						| PostfixOperator::MethodCall(_, _)
						| PostfixOperator::Call(_)
				)
			}
			Expr::Literal(_)
			| Expr::Param(_)
			| Expr::Idiom(_)
			| Expr::Table(_)
			| Expr::Mock(_)
			| Expr::Block(_)
			| Expr::Constant(_)
			| Expr::Prefix { .. }
			| Expr::Binary { .. }
			| Expr::FunctionCall(_) => false,
		}
	}
	/// Returns true if there is a `NONE` or `NULL` value in the left most spot when formatting.
	/// returns true for `NONE + 1`, `NULL()`, `NONE`, `NULL..` etc.
	///
	/// Required for proper formatting when `NONE` can conflict with a clause.
	pub fn has_left_none_null(&self) -> bool {
		match self {
			Expr::Literal(Literal::None) | Expr::Literal(Literal::Null) => true,
			Expr::Binary { left: expr, .. } | Expr::Postfix { expr, .. } => {
				expr.has_left_none_null()
			}
			Expr::Idiom(x) => {
				if let Some(Part::Start(x)) = x.0.first() {
					x.has_left_none_null()
				} else {
					false
				}
			}
			_ => false,
		}
	}
	pub fn has_left_minus(&self) -> bool {
		match self {
			Expr::Prefix {
				op: PrefixOperator::Negate,
				..
			} => true,
			Expr::Postfix { expr, .. } | Expr::Binary { left: expr, .. } => expr.has_left_minus(),
			Expr::Literal(Literal::Integer(x)) => x.is_negative(),
			Expr::Literal(Literal::Float(x)) => x.is_sign_negative(),
			Expr::Literal(Literal::Decimal(x)) => x.is_sign_negative(),
			Expr::Idiom(x) => {
				if let Some(x) = x.0.first()
					&& let Part::Graph(lookup) = x
					&& let LookupKind::Graph(Dir::Out) = lookup.kind
				{
					return true;
				}
				false
			}
			_ => false,
		}
	}
	pub fn has_left_idiom(&self) -> bool {
		match self {
			Expr::Idiom(_) => true,
			Expr::Postfix { expr, .. } | Expr::Binary { left: expr, .. } => expr.has_left_idiom(),
			_ => false,
		}
	}
}
fn convert_public_geometry_to_internal(
	geom: surrealdb_types::Geometry,
) -> crate::compat::val::Geometry {
	match geom {
		surrealdb_types::Geometry::Point(p) => crate::compat::val::Geometry::Point(p),
		surrealdb_types::Geometry::Line(l) => crate::compat::val::Geometry::Line(l),
		surrealdb_types::Geometry::Polygon(p) => crate::compat::val::Geometry::Polygon(p),
		surrealdb_types::Geometry::MultiPoint(mp) => crate::compat::val::Geometry::MultiPoint(mp),
		surrealdb_types::Geometry::MultiLine(ml) => crate::compat::val::Geometry::MultiLine(ml),
		surrealdb_types::Geometry::MultiPolygon(mp) => {
			crate::compat::val::Geometry::MultiPolygon(mp)
		}
		surrealdb_types::Geometry::Collection(c) => crate::compat::val::Geometry::Collection(
			c.into_iter()
				.map(convert_public_geometry_to_internal)
				.collect(),
		),
	}
}
fn convert_public_range_to_literal(range: surrealdb_types::Range) -> Expr {
	use crate::upstream::sql::literal::Literal;
	use crate::upstream::sql::operator::BinaryOperator;
	let range = range.into_inner();
	let op = match (&range.0, &range.1) {
		(std::ops::Bound::Included(_), std::ops::Bound::Included(_)) => {
			BinaryOperator::RangeInclusive
		}
		_ => BinaryOperator::Range,
	};
	let start_expr = match range.0 {
		std::ops::Bound::Included(v) => Expr::from_public_value(v),
		std::ops::Bound::Excluded(v) => Expr::from_public_value(v),
		std::ops::Bound::Unbounded => Expr::Literal(Literal::None),
	};
	let end_expr = match range.1 {
		std::ops::Bound::Included(v) => Expr::from_public_value(v),
		std::ops::Bound::Excluded(v) => Expr::from_public_value(v),
		std::ops::Bound::Unbounded => Expr::Literal(Literal::None),
	};
	Expr::Binary {
		left: Box::new(start_expr),
		op,
		right: Box::new(end_expr),
	}
}
pub fn convert_public_value_to_internal(
	value: surrealdb_types::Value,
) -> crate::compat::val::Value {
	match value {
		surrealdb_types::Value::None => crate::compat::val::Value::None,
		surrealdb_types::Value::Null => crate::compat::val::Value::Null,
		surrealdb_types::Value::Bool(b) => crate::compat::val::Value::Bool(b),
		surrealdb_types::Value::Number(n) => match n {
			surrealdb_types::Number::Int(i) => {
				crate::compat::val::Value::Number(crate::compat::val::Number::Int(i))
			}
			surrealdb_types::Number::Float(f) => {
				crate::compat::val::Value::Number(crate::compat::val::Number::Float(f))
			}
			surrealdb_types::Number::Decimal(d) => {
				crate::compat::val::Value::Number(crate::compat::val::Number::Decimal(d))
			}
		},
		surrealdb_types::Value::String(s) => crate::compat::val::Value::String(s),
		surrealdb_types::Value::Duration(d) => crate::compat::val::Value::Duration(d),
		surrealdb_types::Value::Datetime(dt) => crate::compat::val::Value::Datetime(dt),
		surrealdb_types::Value::Uuid(u) => crate::compat::val::Value::Uuid(u),
		surrealdb_types::Value::Array(a) => {
			crate::compat::val::Value::Array(crate::compat::val::Array::from(
				a.into_iter()
					.map(convert_public_value_to_internal)
					.collect::<Vec<_>>(),
			))
		}
		surrealdb_types::Value::Set(s) => {
			crate::compat::val::Value::Set(crate::compat::val::Set::from(
				s.into_iter()
					.map(convert_public_value_to_internal)
					.collect::<std::collections::BTreeSet<_>>(),
			))
		}
		surrealdb_types::Value::Object(o) => {
			crate::compat::val::Value::Object(crate::compat::val::Object::from(
				o.into_iter()
					.map(|(k, v)| (k, convert_public_value_to_internal(v)))
					.collect::<std::collections::BTreeMap<_, _>>(),
			))
		}
		surrealdb_types::Value::Geometry(g) => {
			crate::compat::val::Value::Geometry(convert_public_geometry_to_internal(g))
		}
		surrealdb_types::Value::Bytes(b) => crate::compat::val::Value::Bytes(b),
		surrealdb_types::Value::Table(t) => crate::compat::val::Value::Table(t.into()),
		surrealdb_types::Value::RecordId(PublicRecordId { table, key }) => {
			let key = convert_public_record_id_key_to_internal(key);
			crate::compat::val::Value::RecordId(crate::compat::val::RecordId {
				table: table.into(),
				key,
			})
		}
		surrealdb_types::Value::File(f) => {
			crate::compat::val::Value::File(crate::compat::val::File {
				bucket: f.bucket,
				key: f.key,
			})
		}
		surrealdb_types::Value::Range(r) => {
			crate::compat::val::Value::Range(Box::new(crate::compat::val::Range {
				start: match r.start {
					Bound::Included(v) => Bound::Included(convert_public_value_to_internal(v)),
					Bound::Excluded(v) => Bound::Excluded(convert_public_value_to_internal(v)),
					Bound::Unbounded => Bound::Unbounded,
				},
				end: match r.end {
					Bound::Included(v) => Bound::Included(convert_public_value_to_internal(v)),
					Bound::Excluded(v) => Bound::Excluded(convert_public_value_to_internal(v)),
					Bound::Unbounded => Bound::Unbounded,
				},
			}))
		}
		surrealdb_types::Value::Regex(r) => crate::compat::val::Value::Regex(r),
	}
}
fn convert_public_record_id_key_to_internal(
	key: surrealdb_types::RecordIdKey,
) -> crate::compat::val::RecordIdKey {
	match key {
		surrealdb_types::RecordIdKey::Number(n) => crate::compat::val::RecordIdKey::Number(n),
		surrealdb_types::RecordIdKey::String(s) => crate::compat::val::RecordIdKey::String(s),
		surrealdb_types::RecordIdKey::Uuid(u) => crate::compat::val::RecordIdKey::Uuid(u),
		surrealdb_types::RecordIdKey::Array(a) => {
			crate::compat::val::RecordIdKey::Array(crate::compat::val::Array::from(
				a.into_iter()
					.map(convert_public_value_to_internal)
					.collect::<Vec<_>>(),
			))
		}
		surrealdb_types::RecordIdKey::Object(o) => {
			crate::compat::val::RecordIdKey::Object(crate::compat::val::Object::from(
				o.into_iter()
					.map(|(k, v)| (k, convert_public_value_to_internal(v)))
					.collect::<std::collections::BTreeMap<_, _>>(),
			))
		}
		surrealdb_types::RecordIdKey::Range(r) => {
			crate::compat::val::RecordIdKey::Range(Box::new(crate::compat::val::RecordIdKeyRange {
				start: match r.start {
					Bound::Included(k) => {
						Bound::Included(convert_public_record_id_key_to_internal(k))
					}
					Bound::Excluded(k) => {
						Bound::Excluded(convert_public_record_id_key_to_internal(k))
					}
					Bound::Unbounded => Bound::Unbounded,
				},
				end: match r.end {
					Bound::Included(k) => {
						Bound::Included(convert_public_record_id_key_to_internal(k))
					}
					Bound::Excluded(k) => {
						Bound::Excluded(convert_public_record_id_key_to_internal(k))
					}
					Bound::Unbounded => Bound::Unbounded,
				},
			}))
		}
	}
}
impl ToSql for Expr {
	fn fmt_sql(&self, f: &mut String, fmt: SqlFormat) {
		match self {
			Expr::Literal(literal) => literal.fmt_sql(f, fmt),
			Expr::Param(param) => param.fmt_sql(f, fmt),
			Expr::Idiom(idiom) => idiom.fmt_sql(f, fmt),
			Expr::Table(ident) => write_sql!(f, fmt, "{}", EscapeIdent(ident)),
			Expr::Mock(mock) => mock.fmt_sql(f, fmt),
			Expr::Block(block) => block.fmt_sql(f, fmt),
			Expr::Constant(constant) => constant.fmt_sql(f, fmt),
			Expr::Prefix { op, expr } => {
				let expr_bp = BindingPower::for_expr(expr);
				let op_bp = BindingPower::for_prefix_operator(op);
				if expr.needs_parentheses()
					|| expr_bp < op_bp
					|| expr_bp == op_bp && matches!(expr_bp, BindingPower::Range)
					|| *op == PrefixOperator::Negate && expr.has_left_minus()
				{
					write_sql!(f, fmt, "{op}({expr})");
				} else {
					write_sql!(f, fmt, "{op}{expr}");
				}
			}
			Expr::Postfix { expr, op } => {
				let expr_bp = BindingPower::for_expr(expr);
				let op_bp = BindingPower::for_postfix_operator(op);
				if expr.needs_parentheses()
					|| expr_bp < op_bp
					|| expr_bp == op_bp && matches!(expr_bp, BindingPower::Range)
					|| matches!(op, PostfixOperator::Call(_))
				{
					write_sql!(f, fmt, "({expr}){op}");
				} else {
					write_sql!(f, fmt, "{expr}{op}");
				}
			}
			Expr::Binary { left, op, right } => {
				let op_bp = BindingPower::for_binary_operator(op);
				let left_bp = BindingPower::for_expr(left);
				let right_bp = BindingPower::for_expr(right);
				if left.needs_parentheses()
					|| left_bp < op_bp
					|| left_bp == op_bp
						&& matches!(
							left_bp,
							BindingPower::Range | BindingPower::Relation | BindingPower::Equality
						) {
					write_sql!(f, fmt, "({left})");
				} else {
					write_sql!(f, fmt, "{left}");
				}
				if matches!(
					op,
					BinaryOperator::Range
						| BinaryOperator::RangeSkip
						| BinaryOperator::RangeInclusive
						| BinaryOperator::RangeSkipInclusive
				) {
					op.fmt_sql(f, fmt);
				} else {
					f.push(' ');
					op.fmt_sql(f, fmt);
					f.push(' ');
				}
				if right.needs_parentheses()
					|| right_bp < op_bp
					|| right_bp == op_bp
						&& matches!(
							right_bp,
							BindingPower::Range | BindingPower::Relation | BindingPower::Equality
						) {
					write_sql!(f, fmt, "({right})");
				} else {
					write_sql!(f, fmt, "{right}");
				}
			}
			Expr::FunctionCall(function_call) => function_call.fmt_sql(f, fmt),
			Expr::Closure(closure) => closure.fmt_sql(f, fmt),
			Expr::Break => f.push_str("BREAK"),
			Expr::Continue => f.push_str("CONTINUE"),
			Expr::Return(x) => x.fmt_sql(f, fmt),
			Expr::Throw(expr) => {
				write_sql!(f, fmt, "THROW {}", CoverStmts(expr.as_ref()))
			}
			Expr::IfElse(s) => s.fmt_sql(f, fmt),
			Expr::Select(s) => s.fmt_sql(f, fmt),
			Expr::Create(s) => s.fmt_sql(f, fmt),
			Expr::Update(s) => s.fmt_sql(f, fmt),
			Expr::Delete(s) => s.fmt_sql(f, fmt),
			Expr::Relate(s) => s.fmt_sql(f, fmt),
			Expr::Insert(s) => s.fmt_sql(f, fmt),
			Expr::Define(s) => s.fmt_sql(f, fmt),
			Expr::Remove(s) => s.fmt_sql(f, fmt),
			Expr::Rebuild(s) => s.fmt_sql(f, fmt),
			Expr::Upsert(s) => s.fmt_sql(f, fmt),
			Expr::Alter(s) => s.fmt_sql(f, fmt),
			Expr::Info(s) => s.fmt_sql(f, fmt),
			Expr::Foreach(s) => s.fmt_sql(f, fmt),
			Expr::Let(s) => s.fmt_sql(f, fmt),
			Expr::Sleep(s) => s.fmt_sql(f, fmt),
			Expr::Explain {
				format: explain_format,
				analyze,
				statement,
			} => {
				f.push_str("EXPLAIN");
				if *analyze {
					f.push_str(" ANALYZE");
				}
				match explain_format {
					ExplainFormat::Text => f.push_str(" FORMAT TEXT"),
					ExplainFormat::Json => f.push_str(" FORMAT JSON"),
				}
				f.push(' ');
				statement.fmt_sql(f, fmt);
			}
		}
	}
}
