use crate::upstream::sql::Param;
use crate::upstream::sql::language::Language;
use crate::upstream::syn::error::bail;
use crate::upstream::syn::lexer::Lexer;
use crate::upstream::syn::lexer::compound::{self, ParsedInt};
use crate::upstream::syn::parser::mac::unexpected;
use crate::upstream::syn::parser::{ParseResult, Parser};
use crate::upstream::syn::token::{Span, TokenKind, t};
use rust_decimal::Decimal;
mod number;
/// A trait for parsing single tokens with a specific value.
pub trait TokenValue: Sized {
	fn from_token(parser: &mut Parser<'_>) -> ParseResult<Self>;
}
impl TokenValue for Language {
	fn from_token(parser: &mut Parser<'_>) -> ParseResult<Self> {
		let peek = parser.peek();
		match peek.kind {
			TokenKind::Language(x) => {
				parser.pop_peek();
				Ok(x)
			}
			t!("NO") => {
				parser.pop_peek();
				Ok(Language::Norwegian)
			}
			_ => unexpected!(parser, peek, "a language"),
		}
	}
}
impl TokenValue for Param {
	fn from_token(parser: &mut Parser<'_>) -> ParseResult<Self> {
		let peek = parser.peek();
		match peek.kind {
			TokenKind::Parameter => {
				parser.pop_peek();
				let mut span = peek.span;
				span.offset += 1;
				span.len -= 1;
				let ident = parser.unescape_ident_span(span)?;
				Ok(Param::new(ident.to_owned()))
			}
			_ => unexpected!(parser, peek, "a parameter"),
		}
	}
}
impl TokenValue for surrealdb_types::Duration {
	fn from_token(parser: &mut Parser<'_>) -> ParseResult<Self> {
		let token = parser.peek();
		match token.kind {
			TokenKind::Digits => {
				parser.pop_peek();
				let v = parser.lexer.lex_compound(token, compound::duration)?.value;
				Ok(surrealdb_types::Duration::from(v))
			}
			_ => unexpected!(parser, token, "a duration"),
		}
	}
}
impl TokenValue for surrealdb_types::Datetime {
	fn from_token(parser: &mut Parser<'_>) -> ParseResult<Self> {
		let token = parser.peek();
		match token.kind {
			t!("d\"") | t!("d'") => {
				parser.pop_peek();
				let str = parser.unescape_string_span(token.span)?;
				let file = Lexer::lex_datetime(str).map_err(|e| {
					e.update_spans(|span| {
						let range = span.to_range();
						let start =
							Lexer::escaped_string_offset(parser.span_str(token.span), range.start);
						let end =
							Lexer::escaped_string_offset(parser.span_str(token.span), range.end);
						*span = Span::from_range(
							(token.span.offset + start)..(token.span.offset + end),
						);
					})
				})?;
				Ok(file)
			}
			_ => unexpected!(parser, token, "a datetime"),
		}
	}
}
impl TokenValue for surrealdb_types::Uuid {
	fn from_token(parser: &mut Parser<'_>) -> ParseResult<Self> {
		let token = parser.peek();
		match token.kind {
			t!("u\"") | t!("u'") => {
				parser.pop_peek();
				let str = parser.unescape_string_span(token.span)?;
				let file = Lexer::lex_uuid(str).map_err(|e| {
					e.update_spans(|span| {
						let range = span.to_range();
						let start =
							Lexer::escaped_string_offset(parser.span_str(token.span), range.start);
						let end =
							Lexer::escaped_string_offset(parser.span_str(token.span), range.end);
						*span = Span::from_range(
							(token.span.offset + start)..(token.span.offset + end),
						);
					})
				})?;
				Ok(file)
			}
			_ => unexpected!(parser, token, "a uuid"),
		}
	}
}
impl TokenValue for surrealdb_types::File {
	fn from_token(parser: &mut Parser<'_>) -> ParseResult<Self> {
		let token = parser.peek();
		if !parser.settings.files_enabled {
			unexpected!(
				parser,
				token,
				"the experimental files feature to be enabled"
			);
		}
		match token.kind {
			t!("f\"") | t!("f'") => {
				parser.pop_peek();
				let str = parser.unescape_string_span(token.span)?;
				let file = Lexer::lex_file(str).map_err(|e| {
					e.update_spans(|span| {
						let range = span.to_range();
						let start =
							Lexer::escaped_string_offset(parser.span_str(token.span), range.start);
						let end =
							Lexer::escaped_string_offset(parser.span_str(token.span), range.end);
						*span = Span::from_range(
							(token.span.offset + start)..(token.span.offset + end),
						);
					})
				})?;
				Ok(file)
			}
			_ => unexpected!(parser, token, "a file"),
		}
	}
}
impl TokenValue for surrealdb_types::Bytes {
	fn from_token(parser: &mut Parser<'_>) -> ParseResult<Self> {
		let token = parser.peek();
		match token.kind {
			t!("b\"") | t!("b'") => {
				parser.pop_peek();
				let str = parser.unescape_string_span(token.span)?;
				let bytes = Lexer::lex_bytes(str).map_err(|e| {
					e.update_spans(|span| {
						let range = span.to_range();
						let start =
							Lexer::escaped_string_offset(parser.span_str(token.span), range.start);
						let end =
							Lexer::escaped_string_offset(parser.span_str(token.span), range.end);
						*span = Span::from_range(
							(token.span.offset + start)..(token.span.offset + end),
						);
					})
				})?;
				Ok(bytes)
			}
			_ => unexpected!(parser, token, "a bytestring"),
		}
	}
}
impl TokenValue for surrealdb_types::Regex {
	fn from_token(parser: &mut Parser<'_>) -> ParseResult<Self> {
		let peek = parser.peek();
		match peek.kind {
			t!("/") => {
				parser.pop_peek();
				if parser.has_peek() {
					parser.backup_after(peek.span);
				}
				let token = parser.lex_compound(peek, compound::regex)?;
				let s = parser.unescape_regex_span(token.span)?;
				match regex::Regex::new(s) {
					Ok(x) => Ok(surrealdb_types::Regex::from(x)),
					Err(e) => {
						bail!("Invalid regex syntax {e}", @ token.span);
					}
				}
			}
			_ => unexpected!(parser, peek, "a regex"),
		}
	}
}
pub enum NumberToken {
	Float(f64),
	Integer(ParsedInt),
	Decimal(Decimal),
}
impl TokenValue for NumberToken {
	fn from_token(parser: &mut Parser<'_>) -> ParseResult<Self> {
		let token = parser.peek();
		match token.kind {
			t!("+") | t!("-") | TokenKind::Digits => {
				parser.pop_peek();
				let token = parser.lex_compound(token, compound::number)?;
				match token.value {
					compound::Numeric::Float(f) => Ok(NumberToken::Float(f)),
					compound::Numeric::Integer(x) => Ok(NumberToken::Integer(x)),
					compound::Numeric::Decimal(d) => Ok(NumberToken::Decimal(d)),
					compound::Numeric::Duration(_) => {
						bail!(
							"Unexpected token `duration`, expected a number", @ token
							.span
						)
					}
				}
			}
			TokenKind::NaN => {
				parser.pop_peek();
				Ok(NumberToken::Float(f64::NAN))
			}
			TokenKind::Infinity => {
				parser.pop_peek();
				Ok(NumberToken::Float(f64::INFINITY))
			}
			_ => unexpected!(parser, token, "a number"),
		}
	}
}
impl TokenValue for surrealdb_types::Number {
	fn from_token(parser: &mut Parser<'_>) -> ParseResult<Self> {
		let token = parser.next_token_value::<NumberToken>()?;
		match token {
			NumberToken::Float(x) => Ok(Self::Float(x)),
			NumberToken::Integer(i) => Ok(Self::Int(i.into_int(parser.recent_span())?)),
			NumberToken::Decimal(x) => Ok(Self::Decimal(x)),
		}
	}
}
impl Parser<'_> {
	/// Parse a token value from the next token in the parser.
	pub fn next_token_value<V: TokenValue>(&mut self) -> ParseResult<V> {
		V::from_token(self)
	}
	pub fn parse_string_lit(&mut self) -> ParseResult<String> {
		let token = self.peek();
		match token.kind {
			t!("\"") | t!("'") => {
				self.pop_peek();
				let str = self.unescape_string_span(token.span)?;
				Ok(str.to_owned())
			}
			_ => unexpected!(self, token, "a strand"),
		}
	}
	pub fn parse_ident(&mut self) -> ParseResult<String> {
		self.parse_ident_str().map(|x| x.to_owned())
	}
	pub fn parse_ident_str(&mut self) -> ParseResult<&str> {
		let token = self.next();
		match token.kind {
			TokenKind::Identifier => self.unescape_ident_span(token.span),
			x if Self::kind_is_keyword_like(x) => Ok(self.span_str(token.span)),
			_ => {
				unexpected!(self, token, "an identifier");
			}
		}
	}
	pub fn parse_flexible_ident(&mut self) -> ParseResult<String> {
		let token = self.next();
		match token.kind {
			TokenKind::Digits => {
				let span = if let Some(peek) = self.peek_whitespace() {
					match peek.kind {
						x if Self::kind_is_keyword_like(x) => {
							self.pop_peek();
							token.span.covers(peek.span)
						}
						TokenKind::Identifier | TokenKind::NaN | TokenKind::Infinity => {
							self.pop_peek();
							token.span.covers(peek.span)
						}
						_ => token.span,
					}
				} else {
					token.span
				};
				Ok(self.span_str(span).to_owned())
			}
			TokenKind::Identifier | TokenKind::NaN | TokenKind::Infinity => {
				let str = self.unescape_ident_span(token.span)?;
				Ok(str.to_owned())
			}
			x if Self::kind_is_keyword_like(x) => Ok(self.span_str(token.span).to_owned()),
			_ => {
				unexpected!(self, token, "an identifier");
			}
		}
	}
}
