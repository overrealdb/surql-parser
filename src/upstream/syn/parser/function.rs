use super::{ParseResult, Parser};
use crate::upstream::sql::{Expr, Function, FunctionCall, Model};
use crate::upstream::syn::error::{bail, syntax_error};
use crate::upstream::syn::parser::mac::{expected, expected_whitespace, unexpected};
use crate::upstream::syn::token::{TokenKind, t};
use reblessive::Stk;
impl Parser<'_> {
	pub async fn parse_function_name(&mut self) -> ParseResult<Function> {
		let peek = self.peek();
		let fnc = match peek.kind {
			t!("fn") => {
				self.pop_peek();
				expected!(self, t!("::"));
				let mut name = self.parse_ident()?;
				while self.eat(t!("::")) {
					name.push_str("::");
					name.push_str(self.parse_ident_str()?);
				}
				Function::Custom(name)
			}
			t!("mod") => {
				self.pop_peek();
				if !self.settings.surrealism_enabled {
					bail!(
						"Experimental capability `surrealism` is not enabled", @ self
						.last_span() => "Use of `mod::` is still experimental"
					)
				}
				expected!(self, t!("::"));
				let name = self.parse_ident()?;
				let sub = if self.eat(t!("::")) {
					Some(self.parse_ident()?)
				} else {
					None
				};
				Function::Module(name, sub)
			}
			t!("silo") => {
				self.pop_peek();
				if !self.settings.surrealism_enabled {
					bail!(
						"Experimental capability `surrealism` is not enabled", @ self
						.last_span() => "Use of `silo::` is still experimental"
					)
				}
				expected!(self, t!("::"));
				let org = self.parse_ident()?;
				expected!(self, t!("::"));
				let pkg = self.parse_ident()?;
				expected!(self, t!("<"));
				let major = self.parse_version_digits()?;
				expected!(self, t!("."));
				let minor = self.parse_version_digits()?;
				expected!(self, t!("."));
				let patch = self.parse_version_digits()?;
				expected!(self, t!(">"));
				let sub = if self.eat(t!("::")) {
					Some(self.parse_ident()?)
				} else {
					None
				};
				Function::Silo {
					org,
					pkg,
					major,
					minor,
					patch,
					sub,
				}
			}
			t!("ml") => {
				self.pop_peek();
				expected!(self, t!("::"));
				let mut name = self.parse_ident()?;
				while self.eat(t!("::")) {
					name.push_str("::");
					name.push_str(self.parse_ident_str()?);
				}
				let (major, minor, patch) = self.parse_model_version()?;
				let version = format!("{}.{}.{}", major, minor, patch);
				Function::Model(Model { name, version })
			}
			TokenKind::Identifier => {
				let mut name = self.parse_ident()?;
				while self.eat(t!("::")) {
					name.push_str("::");
					name.push_str(self.parse_ident_str()?)
				}
				Function::Normal(name)
			}
			x if Self::kind_is_keyword_like(x) => {
				self.pop_peek();
				let mut name = self.lexer.span_str(peek.span).to_string();
				while self.eat(t!("::")) {
					name.push_str("::");
					name.push_str(self.parse_ident_str()?)
				}
				Function::Normal(name)
			}
			_ => unexpected!(self, self.peek(), "a function name"),
		};
		Ok(fnc)
	}
	/// Parse a custom function function call
	///
	/// Expects `fn` to already be called.
	pub(super) async fn parse_custom_function(
		&mut self,
		stk: &mut Stk,
	) -> ParseResult<FunctionCall> {
		expected!(self, t!("::"));
		let mut name = self.parse_ident()?;
		while self.eat(t!("::")) {
			name.push_str("::");
			name.push_str(self.parse_ident_str()?)
		}
		expected!(self, t!("(")).span;
		let args = self.parse_function_args(stk).await?;
		let name = Function::Custom(name);
		Ok(FunctionCall {
			receiver: name,
			arguments: args,
		})
	}
	/// Parse a module function function call
	///
	/// Expects `mod` to already be called.
	pub(super) async fn parse_module_function(
		&mut self,
		stk: &mut Stk,
	) -> ParseResult<FunctionCall> {
		if !self.settings.surrealism_enabled {
			bail!(
				"Experimental capability `surrealism` is not enabled", @ self.last_span()
				=> "Use of `mod::` is still experimental"
			)
		}
		expected!(self, t!("::"));
		let name = self.parse_ident()?;
		let sub = if self.eat(t!("::")) {
			Some(self.parse_ident()?)
		} else {
			None
		};
		expected!(self, t!("(")).span;
		let args = self.parse_function_args(stk).await?;
		let name = Function::Module(name, sub);
		Ok(FunctionCall {
			receiver: name,
			arguments: args,
		})
	}
	/// Parse a silo function function call
	///
	/// Expects `silo` to already be called.
	pub(super) async fn parse_silo_function(&mut self, stk: &mut Stk) -> ParseResult<FunctionCall> {
		if !self.settings.surrealism_enabled {
			bail!(
				"Experimental capability `surrealism` is not enabled", @ self.last_span()
				=> "Use of `silo::` is still experimental"
			)
		}
		expected!(self, t!("::"));
		let org = self.parse_ident()?;
		expected!(self, t!("::"));
		let pkg = self.parse_ident()?;
		expected!(self, t!("<"));
		let major = self.parse_version_digits()?;
		expected!(self, t!("."));
		let minor = self.parse_version_digits()?;
		expected!(self, t!("."));
		let patch = self.parse_version_digits()?;
		expected!(self, t!(">"));
		let sub = if self.eat(t!("::")) {
			Some(self.parse_ident()?)
		} else {
			None
		};
		expected!(self, t!("(")).span;
		let args = self.parse_function_args(stk).await?;
		let name = Function::Silo {
			org,
			pkg,
			major,
			minor,
			patch,
			sub,
		};
		Ok(FunctionCall {
			receiver: name,
			arguments: args,
		})
	}
	pub(super) async fn parse_function_args(&mut self, stk: &mut Stk) -> ParseResult<Vec<Expr>> {
		let start = self.last_span();
		let mut args = Vec::new();
		loop {
			if self.eat(t!(")")) {
				break;
			}
			let arg = stk.run(|ctx| self.parse_expr_inherit(ctx)).await?;
			args.push(arg);
			if !self.eat(t!(",")) {
				self.expect_closing_delimiter(t!(")"), start)?;
				break;
			}
		}
		Ok(args)
	}
	pub fn parse_version_digits(&mut self) -> ParseResult<u32> {
		let token = self.next();
		match token.kind {
			TokenKind::Digits => self
				.span_str(token.span)
				.parse::<u32>()
				.map_err(|e| syntax_error!("Failed to parse model version: {e}", @ token.span)),
			_ => unexpected!(self, token, "an integer"),
		}
	}
	pub(super) fn parse_model_version(&mut self) -> ParseResult<(u32, u32, u32)> {
		let start = expected!(self, t!("<")).span;
		let major: u32 = self.parse_version_digits()?;
		expected_whitespace!(self, t!("."));
		let minor: u32 = self.parse_version_digits()?;
		expected_whitespace!(self, t!("."));
		let patch: u32 = self.parse_version_digits()?;
		self.expect_closing_delimiter(t!(">"), start)?;
		Ok((major, minor, patch))
	}
	/// Parse a model invocation
	///
	/// Expects `ml` to already be called.
	pub(super) async fn parse_model(&mut self, stk: &mut Stk) -> ParseResult<FunctionCall> {
		expected!(self, t!("::"));
		let mut name = self.parse_ident()?;
		while self.eat(t!("::")) {
			name.push_str("::");
			name.push_str(self.parse_ident_str()?)
		}
		let (major, minor, patch) = self.parse_model_version()?;
		let start = expected!(self, t!("(")).span;
		let mut args = Vec::new();
		loop {
			if self.eat(t!(")")) {
				break;
			}
			let arg = stk.run(|ctx| self.parse_expr_inherit(ctx)).await?;
			args.push(arg);
			if !self.eat(t!(",")) {
				self.expect_closing_delimiter(t!(")"), start)?;
				break;
			}
		}
		let func = Function::Model(Model {
			name,
			version: format!("{}.{}.{}", major, minor, patch),
		});
		Ok(FunctionCall {
			receiver: func,
			arguments: args,
		})
	}
}
