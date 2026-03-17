use super::{ParseResult, Parser};
use crate::upstream::sql::lookup::LookupSubject;
use crate::upstream::sql::{
	Param, RecordIdKeyGen, RecordIdKeyLit, RecordIdKeyRangeLit, RecordIdLit,
};
use crate::upstream::syn::error::bail;
use crate::upstream::syn::lexer::compound;
use crate::upstream::syn::parser::mac::{expected, expected_whitespace, unexpected};
use crate::upstream::syn::token::{TokenKind, t};
use reblessive::Stk;
use std::cmp::Ordering;
use std::ops::Bound;
use surrealdb_types::ToSql;
impl Parser<'_> {
	pub async fn parse_record_id_or_range(
		&mut self,
		stk: &mut Stk,
		ident: String,
	) -> ParseResult<RecordIdLit> {
		expected_whitespace!(self, t!(":"));
		if self.eat_whitespace(t!("..")) {
			let end = if self.eat_whitespace(t!("=")) {
				let id = stk.run(|stk| self.parse_record_id_key(stk)).await?;
				Bound::Included(id)
			} else if let Some(peek) = self.peek_whitespace()
				&& Self::kind_starts_record_id_key(peek.kind)
			{
				let id = stk.run(|stk| self.parse_record_id_key(stk)).await?;
				Bound::Excluded(id)
			} else {
				Bound::Unbounded
			};
			return Ok(RecordIdLit {
				table: ident,
				key: RecordIdKeyLit::Range(Box::new(RecordIdKeyRangeLit {
					start: Bound::Unbounded,
					end,
				})),
			});
		}
		let beg = if let Some(peek) = self.peek_whitespace()
			&& Self::kind_starts_record_id_key(peek.kind)
		{
			let v = stk.run(|stk| self.parse_record_id_key(stk)).await?;
			if self.eat_whitespace(t!(">")) {
				Bound::Excluded(v)
			} else {
				Bound::Included(v)
			}
		} else {
			Bound::Unbounded
		};
		if self.eat_whitespace(t!("..")) {
			let end = if self.eat_whitespace(t!("=")) {
				let id = stk.run(|stk| self.parse_record_id_key(stk)).await?;
				Bound::Included(id)
			} else if let Some(peek) = self.peek_whitespace()
				&& Self::kind_starts_record_id_key(peek.kind)
			{
				let id = stk.run(|stk| self.parse_record_id_key(stk)).await?;
				Bound::Excluded(id)
			} else {
				Bound::Unbounded
			};
			Ok(RecordIdLit {
				table: ident,
				key: RecordIdKeyLit::Range(Box::new(RecordIdKeyRangeLit { start: beg, end })),
			})
		} else {
			let id = match beg {
				Bound::Unbounded => {
					if let Some(token) = self.peek_whitespace()
						&& token.kind == t!("$param")
					{
						let param = self.next_token_value::<Param>()?;
						bail!(
							"Unexpected token `$param` expected a record-id key", @ token
							.span =>
							"Record-id's can be create from a param with `type::record(\"{}\",{})`",
							ident, param.to_sql()
						);
					}
					unexpected!(self, self.peek(), "a record-id key")
				}
				Bound::Excluded(_) => {
					unexpected!(self, self.peek(), "the range operator `..`")
				}
				Bound::Included(v) => v,
			};
			Ok(RecordIdLit {
				table: ident,
				key: id,
			})
		}
	}
	pub async fn parse_id_range(&mut self, stk: &mut Stk) -> ParseResult<RecordIdKeyRangeLit> {
		let beg = if let Some(peek) = self.peek_whitespace()
			&& Self::kind_starts_record_id_key(peek.kind)
		{
			let v = stk.run(|stk| self.parse_record_id_key(stk)).await?;
			if self.eat_whitespace(t!(">")) {
				Bound::Excluded(v)
			} else {
				Bound::Included(v)
			}
		} else {
			Bound::Unbounded
		};
		expected!(self, t!(".."));
		let end = if self.eat_whitespace(t!("=")) {
			let id = stk.run(|stk| self.parse_record_id_key(stk)).await?;
			Bound::Included(id)
		} else if let Some(peek) = self.peek_whitespace()
			&& Self::kind_starts_record_id_key(peek.kind)
		{
			let id = stk.run(|stk| self.parse_record_id_key(stk)).await?;
			Bound::Excluded(id)
		} else {
			Bound::Unbounded
		};
		Ok(RecordIdKeyRangeLit { start: beg, end })
	}
	pub async fn parse_lookup_subject(
		&mut self,
		stk: &mut Stk,
		supports_referencing_field: bool,
	) -> ParseResult<LookupSubject> {
		let table = self.parse_ident()?;
		if self.eat_whitespace(t!(":")) {
			let range = self.parse_id_range(stk).await?;
			let referencing_field = self
				.parse_referencing_field(supports_referencing_field)
				.await?;
			Ok(LookupSubject::Range {
				table,
				range,
				referencing_field,
			})
		} else {
			Ok(LookupSubject::Table {
				table,
				referencing_field: self
					.parse_referencing_field(supports_referencing_field)
					.await?,
			})
		}
	}
	pub async fn parse_referencing_field(
		&mut self,
		supports_referencing_field: bool,
	) -> ParseResult<Option<String>> {
		if supports_referencing_field && self.eat(t!("FIELD")) {
			Ok(Some(self.parse_ident()?))
		} else {
			Ok(None)
		}
	}
	pub async fn parse_record_id(&mut self, stk: &mut Stk) -> ParseResult<RecordIdLit> {
		let ident = self.parse_ident()?;
		self.parse_record_id_from_ident(stk, ident).await
	}
	pub async fn parse_record_id_from_ident(
		&mut self,
		stk: &mut Stk,
		ident: String,
	) -> ParseResult<RecordIdLit> {
		expected!(self, t!(":"));
		let id = stk.run(|ctx| self.parse_record_id_key(ctx)).await?;
		Ok(RecordIdLit {
			table: ident,
			key: id,
		})
	}
	pub async fn parse_record_id_key(&mut self, stk: &mut Stk) -> ParseResult<RecordIdKeyLit> {
		let Some(token) = self.peek_whitespace() else {
			bail!("Unexpected whitespace after record-id table", @ self.peek().span)
		};
		match token.kind {
			t!("u'") | t!("u\"") => Ok(RecordIdKeyLit::Uuid(self.next_token_value()?)),
			t!("{") => {
				self.pop_peek();
				let object = self.parse_object(stk, token.span).await?;
				Ok(RecordIdKeyLit::Object(object))
			}
			t!("[") => {
				self.pop_peek();
				let array = self.parse_array(stk, token.span).await?;
				Ok(RecordIdKeyLit::Array(array))
			}
			t!("+") => {
				self.pop_peek();
				let digits_token = if let Some(digits_token) = self.peek_whitespace() {
					match digits_token.kind {
						TokenKind::Digits => digits_token,
						_ => unexpected!(self, digits_token, "an integer"),
					}
				} else {
					unexpected!(self, token, "a record-id key")
				};
				if let Some(next) = self.peek_whitespace() {
					match next.kind {
						t!(".") => {
							unexpected!(
								self, next, "an integer", =>
								"Numeric Record-id keys can only be integers"
							);
						}
						x if Self::kind_is_identifier(x) => {
							let span = token.span.covers(next.span);
							bail!("Unexpected token `{x}` expected an integer", @ span);
						}
						_ => {}
					}
				}
				let digits_str = self.span_str(digits_token.span);
				if let Ok(number) = digits_str.parse() {
					Ok(RecordIdKeyLit::Number(number))
				} else {
					Ok(RecordIdKeyLit::String(digits_str.to_owned()))
				}
			}
			t!("-") => {
				self.pop_peek();
				let token = expected!(self, TokenKind::Digits);
				if let Ok(number) = self.lex_compound(token, compound::integer::<u64>) {
					match number.value.cmp(&((i64::MAX as u64) + 1)) {
						Ordering::Less => Ok(RecordIdKeyLit::Number(-(number.value as i64))),
						Ordering::Equal => Ok(RecordIdKeyLit::Number(i64::MIN)),
						Ordering::Greater => Ok(RecordIdKeyLit::String(format!(
							"-{}",
							self.span_str(number.span)
						))),
					}
				} else {
					let strand = format!("-{}", self.span_str(token.span));
					Ok(RecordIdKeyLit::String(strand))
				}
			}
			TokenKind::Digits => {
				if self.settings.flexible_record_id
					&& let Some(next) = self.peek_whitespace1()
					&& (Self::kind_is_identifier(next.kind)
						|| next.kind == TokenKind::NaN
						|| next.kind == TokenKind::Infinity)
				{
					let ident = self.parse_flexible_ident()?;
					return Ok(RecordIdKeyLit::String(ident));
				}
				self.pop_peek();
				let digits_str = self.span_str(token.span);
				if let Ok(number) = digits_str.parse::<i64>() {
					Ok(RecordIdKeyLit::Number(number))
				} else {
					Ok(RecordIdKeyLit::String(digits_str.to_owned()))
				}
			}
			t!("ULID") => {
				let token = self.pop_peek();
				if self.eat(t!("(")) {
					expected!(self, t!(")"));
					Ok(RecordIdKeyLit::Generate(RecordIdKeyGen::Ulid))
				} else {
					let slice = self.span_str(token.span);
					Ok(RecordIdKeyLit::String(slice.to_owned()))
				}
			}
			t!("UUID") => {
				let token = self.pop_peek();
				if self.eat(t!("(")) {
					expected!(self, t!(")"));
					Ok(RecordIdKeyLit::Generate(RecordIdKeyGen::Uuid))
				} else {
					let slice = self.span_str(token.span);
					Ok(RecordIdKeyLit::String(slice.to_owned()))
				}
			}
			t!("RAND") => {
				let token = self.pop_peek();
				if self.eat(t!("(")) {
					expected!(self, t!(")"));
					Ok(RecordIdKeyLit::Generate(RecordIdKeyGen::Rand))
				} else {
					let slice = self.span_str(token.span);
					Ok(RecordIdKeyLit::String(slice.to_owned()))
				}
			}
			_ => {
				let ident = if self.settings.flexible_record_id {
					self.parse_flexible_ident()?
				} else {
					self.parse_ident()?
				};
				Ok(RecordIdKeyLit::String(ident))
			}
		}
	}
}
