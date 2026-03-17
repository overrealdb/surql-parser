use super::mac::unexpected;
use super::{ParseResult, Parser};
use crate::compat::types::PublicDuration;
use crate::upstream::sql::Kind;
use crate::upstream::sql::kind::{GeometryKind, KindLiteral};
use crate::upstream::syn::lexer::compound;
use crate::upstream::syn::parser::mac::expected;
use crate::upstream::syn::token::{Keyword, Span, TokenKind, t};
use reblessive::Stk;
use std::collections::BTreeMap;
impl Parser<'_> {
	/// Parse a kind production.
	///
	/// # Parser State
	/// expects the first `<` to already be eaten
	pub async fn parse_kind(&mut self, stk: &mut Stk, delim: Span) -> ParseResult<Kind> {
		let kind = self.parse_inner_kind(stk).await?;
		self.expect_closing_delimiter(t!(">"), delim)?;
		Ok(kind)
	}
	/// Parse an inner kind, a kind without enclosing `<` `>`.
	pub async fn parse_inner_kind(&mut self, stk: &mut Stk) -> ParseResult<Kind> {
		match self.parse_inner_single_kind(stk).await? {
			Kind::Any => Ok(Kind::Any),
			first => {
				if self.peek_kind() == t!("|") {
					let mut kind = vec![first];
					while self.eat(t!("|")) {
						kind.push(stk.run(|ctx| self.parse_concrete_kind(ctx)).await?);
					}
					let kind = Kind::either(kind);
					Ok(kind)
				} else {
					Ok(first)
				}
			}
		}
	}
	/// Parse a single inner kind, a kind without enclosing `<` `>`.
	pub(super) async fn parse_inner_single_kind(&mut self, stk: &mut Stk) -> ParseResult<Kind> {
		match self.peek_kind() {
			t!("ANY") => {
				self.pop_peek();
				Ok(Kind::Any)
			}
			t!("OPTION") => {
				self.pop_peek();
				let delim = expected!(self, t!("<")).span;
				let mut kinds = vec![
					Kind::None,
					stk.run(|ctx| self.parse_concrete_kind(ctx)).await?,
				];
				if self.peek_kind() == t!("|") {
					while self.eat(t!("|")) {
						kinds.push(stk.run(|ctx| self.parse_concrete_kind(ctx)).await?);
					}
				}
				self.expect_closing_delimiter(t!(">"), delim)?;
				Ok(Kind::either(kinds))
			}
			_ => stk.run(|ctx| self.parse_concrete_kind(ctx)).await,
		}
	}
	/// Parse a single kind which is not any, option, or either.
	async fn parse_concrete_kind(&mut self, stk: &mut Stk) -> ParseResult<Kind> {
		let next = self.next();
		match next.kind {
			t!("true") => Ok(Kind::Literal(KindLiteral::Bool(true))),
			t!("false") => Ok(Kind::Literal(KindLiteral::Bool(false))),
			t!("'") | t!("\"") => {
				let str = self.unescape_string_span(next.span)?;
				Ok(Kind::Literal(KindLiteral::String(str.to_owned())))
			}
			TokenKind::NaN => Ok(Kind::Literal(KindLiteral::Float(f64::NAN))),
			TokenKind::Infinity => Ok(Kind::Literal(KindLiteral::Float(f64::INFINITY))),
			t!("+") | t!("-") => {
				let compound = self.lex_compound(next, compound::number)?;
				let kind = match compound.value {
					compound::Numeric::Float(f) => KindLiteral::Float(f),
					compound::Numeric::Integer(int) => {
						KindLiteral::Integer(int.into_int(compound.span)?)
					}
					compound::Numeric::Decimal(decimal) => KindLiteral::Decimal(decimal),
					compound::Numeric::Duration(_) => unreachable!(),
				};
				Ok(Kind::Literal(kind))
			}
			TokenKind::Digits => {
				let compound = self.lex_compound(next, compound::numeric)?;
				let v = match compound.value {
					compound::Numeric::Integer(x) => {
						KindLiteral::Integer(x.into_int(compound.span)?)
					}
					compound::Numeric::Float(x) => KindLiteral::Float(x),
					compound::Numeric::Decimal(x) => KindLiteral::Decimal(x),
					compound::Numeric::Duration(x) => {
						KindLiteral::Duration(PublicDuration::from_std(x))
					}
				};
				Ok(Kind::Literal(v))
			}
			t!("{") => {
				let mut obj = BTreeMap::new();
				while !self.eat(t!("}")) {
					let key = self.parse_object_key()?;
					expected!(self, t!(":"));
					let kind = stk.run(|ctx| self.parse_inner_kind(ctx)).await?;
					obj.insert(key, kind);
					self.eat(t!(","));
				}
				Ok(Kind::Literal(KindLiteral::Object(obj)))
			}
			t!("[") => {
				let mut arr = Vec::new();
				while !self.eat(t!("]")) {
					let kind = stk.run(|ctx| self.parse_inner_kind(ctx)).await?;
					arr.push(kind);
					self.eat(t!(","));
				}
				Ok(Kind::Literal(KindLiteral::Array(arr)))
			}
			t!("BOOL") => Ok(Kind::Bool),
			t!("NONE") => Ok(Kind::None),
			t!("NULL") => Ok(Kind::Null),
			t!("BYTES") => Ok(Kind::Bytes),
			t!("DATETIME") => Ok(Kind::Datetime),
			t!("DECIMAL") => Ok(Kind::Decimal),
			t!("DURATION") => Ok(Kind::Duration),
			t!("FLOAT") => Ok(Kind::Float),
			t!("INT") => Ok(Kind::Int),
			t!("NUMBER") => Ok(Kind::Number),
			t!("OBJECT") => Ok(Kind::Object),
			t!("POINT") => Ok(Kind::Geometry(vec![GeometryKind::Point])),
			t!("STRING") => Ok(Kind::String),
			t!("UUID") => Ok(Kind::Uuid),
			t!("RANGE") => Ok(Kind::Range),
			t!("REGEX") => Ok(Kind::Regex),
			t!("FUNCTION") => Ok(Kind::Function(Default::default(), Default::default())),
			t!("RECORD") => {
				let span = self.peek().span;
				if self.eat(t!("<")) {
					let mut tables = vec![self.parse_ident()?];
					while self.eat(t!("|")) {
						tables.push(self.parse_ident()?);
					}
					self.expect_closing_delimiter(t!(">"), span)?;
					Ok(Kind::Record(tables))
				} else {
					Ok(Kind::Record(Vec::new()))
				}
			}
			t!("TABLE") => {
				let span = self.peek().span;
				if self.eat(t!("<")) {
					let mut tables = vec![self.parse_ident()?];
					while self.eat(t!("|")) {
						tables.push(self.parse_ident()?);
					}
					self.expect_closing_delimiter(t!(">"), span)?;
					Ok(Kind::Table(tables))
				} else {
					Ok(Kind::Table(Vec::new()))
				}
			}
			t!("GEOMETRY") => {
				let span = self.peek().span;
				if self.eat(t!("<")) {
					let mut kind = vec![self.parse_geometry_kind()?];
					while self.eat(t!("|")) {
						kind.push(self.parse_geometry_kind()?);
					}
					self.expect_closing_delimiter(t!(">"), span)?;
					Ok(Kind::Geometry(kind))
				} else {
					Ok(Kind::Geometry(Vec::new()))
				}
			}
			t!("ARRAY") => {
				let span = self.peek().span;
				if self.eat(t!("<")) {
					let kind = stk.run(|ctx| self.parse_inner_kind(ctx)).await?;
					let size = self
						.eat(t!(","))
						.then(|| self.next_token_value())
						.transpose()?;
					self.expect_closing_delimiter(t!(">"), span)?;
					Ok(Kind::Array(Box::new(kind), size))
				} else {
					Ok(Kind::Array(Box::new(Kind::Any), None))
				}
			}
			t!("SET") => {
				let span = self.peek().span;
				if self.eat(t!("<")) {
					let kind = stk.run(|ctx| self.parse_inner_kind(ctx)).await?;
					let size = self
						.eat(t!(","))
						.then(|| self.next_token_value())
						.transpose()?;
					self.expect_closing_delimiter(t!(">"), span)?;
					Ok(Kind::Set(Box::new(kind), size))
				} else {
					Ok(Kind::Set(Box::new(Kind::Any), None))
				}
			}
			t!("FILE") => {
				let span = self.peek().span;
				if self.eat(t!("<")) {
					let mut buckets = vec![self.parse_ident()?];
					while self.eat(t!("|")) {
						buckets.push(self.parse_ident()?);
					}
					self.expect_closing_delimiter(t!(">"), span)?;
					Ok(Kind::File(buckets))
				} else {
					Ok(Kind::File(Vec::new()))
				}
			}
			_ => unexpected!(self, next, "a kind name"),
		}
	}
	/// Parse the kind of gemoetry
	fn parse_geometry_kind(&mut self) -> ParseResult<GeometryKind> {
		let next = self.next();
		match next.kind {
			TokenKind::Keyword(keyword) => match keyword {
				Keyword::Point => Ok(GeometryKind::Point),
				Keyword::Line => Ok(GeometryKind::Line),
				Keyword::Polygon => Ok(GeometryKind::Polygon),
				Keyword::MultiPoint => Ok(GeometryKind::MultiPoint),
				Keyword::MultiLine => Ok(GeometryKind::MultiLine),
				Keyword::MultiPolygon => Ok(GeometryKind::MultiPolygon),
				Keyword::Collection => Ok(GeometryKind::Collection),
				_ => unexpected!(self, next, "a geometry kind name"),
			},
			_ => unexpected!(self, next, "a geometry kind name"),
		}
	}
}
