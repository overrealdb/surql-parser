//! Determine the table context at a given cursor position in SurrealQL source.
//!
//! Scans backwards through tokens from the cursor to find the most recent
//! statement keyword (SELECT..FROM, UPDATE, CREATE, etc.) and extracts the
//! table name that follows it.

use surql_parser::upstream::syn::lexer::Lexer;
use surql_parser::upstream::syn::token::{Token, TokenKind};
use tower_lsp::lsp_types::Position;

use crate::completion::position_to_byte_offset;

/// Determine which table is being referenced at a cursor position.
///
/// Scans backwards from cursor to find FROM/UPDATE/INSERT INTO/DEFINE FIELD ON/
/// CREATE/DELETE/UPSERT and returns the table identifier that follows.
pub fn table_context_at_position(source: &str, position: Position) -> Option<String> {
	let cursor_byte = position_to_byte_offset(source, position);
	let prefix = &source[..cursor_byte.min(source.len())];
	if prefix.is_empty() {
		return None;
	}

	let bytes = prefix.as_bytes();
	if bytes.len() > u32::MAX as usize {
		return None;
	}

	let tokens: Vec<Token> = match std::panic::catch_unwind(|| Lexer::new(bytes).collect()) {
		Ok(t) => t,
		Err(_) => return None,
	};

	if tokens.is_empty() {
		return None;
	}

	let text_of = |idx: usize| -> &str {
		let tok = &tokens[idx];
		let start = tok.span.offset as usize;
		let end = start + tok.span.len as usize;
		if end <= prefix.len() {
			&prefix[start..end]
		} else {
			""
		}
	};

	let is_keyword = |idx: usize| -> bool { matches!(tokens[idx].kind, TokenKind::Keyword(_)) };

	let is_ident_like = |idx: usize| -> bool {
		matches!(
			tokens[idx].kind,
			TokenKind::Identifier | TokenKind::Keyword(_)
		)
	};

	let last = tokens.len().saturating_sub(1);
	for i in (0..=last).rev() {
		let tok = &tokens[i];

		if is_keyword(i) {
			let upper = text_of(i).to_uppercase();
			match upper.as_str() {
				// FROM <identifier> -- covers SELECT ... FROM table
				"FROM" => return extract_table_name(i, &tokens, prefix),
				// UPDATE <identifier>
				"UPDATE" => return extract_table_name(i, &tokens, prefix),
				// CREATE <identifier>
				"CREATE" => return extract_table_name(i, &tokens, prefix),
				// DELETE <identifier>
				"DELETE" => return extract_table_name(i, &tokens, prefix),
				// UPSERT <identifier>
				"UPSERT" => return extract_table_name(i, &tokens, prefix),
				// INSERT INTO <identifier>
				"INTO" => {
					if i > 0 && is_keyword(i - 1) && text_of(i - 1).eq_ignore_ascii_case("INSERT") {
						return extract_table_name(i, &tokens, prefix);
					}
				}
				// DEFINE FIELD/INDEX/EVENT ... ON <identifier>
				"ON" => {
					for j in (0..i).rev() {
						if is_keyword(j) {
							let kw = text_of(j).to_uppercase();
							match kw.as_str() {
								"FIELD" | "INDEX" | "EVENT" => {
									if j > 0
										&& is_keyword(j - 1) && text_of(j - 1)
										.eq_ignore_ascii_case("DEFINE")
									{
										return extract_table_name(i, &tokens, prefix);
									}
									break;
								}
								"DEFINE" => break,
								_ => continue,
							}
						} else if is_ident_like(j)
							|| matches!(
								tokens[j].kind,
								TokenKind::LeftChefron | TokenKind::RightChefron | TokenKind::Dot
							) {
							continue;
						} else {
							break;
						}
					}
				}
				_ => {}
			}
		}

		// Stop at statement boundaries so we don't match a previous statement.
		if tok.kind == TokenKind::SemiColon {
			break;
		}
	}

	None
}

/// Given a keyword token at index `kw_idx`, return the text of the next
/// identifier-like token (skipping the ONLY keyword if present).
fn extract_table_name(kw_idx: usize, tokens: &[Token], source: &str) -> Option<String> {
	let mut next = kw_idx + 1;

	// Skip past ONLY keyword (e.g. DELETE ... ONLY, UPDATE ... ONLY)
	if next < tokens.len()
		&& matches!(tokens[next].kind, TokenKind::Keyword(_))
		&& token_text(&tokens[next], source).eq_ignore_ascii_case("ONLY")
	{
		next += 1;
	}

	// Also skip past TABLE keyword (e.g. DEFINE FIELD ... ON TABLE user)
	if next < tokens.len()
		&& matches!(tokens[next].kind, TokenKind::Keyword(_))
		&& token_text(&tokens[next], source).eq_ignore_ascii_case("TABLE")
	{
		next += 1;
	}

	if next < tokens.len() {
		let tok = &tokens[next];
		match tok.kind {
			TokenKind::Identifier | TokenKind::Keyword(_) => {
				let text = token_text(tok, source);
				let text = text.strip_prefix('`').unwrap_or(text);
				let text = text.strip_suffix('`').unwrap_or(text);
				if !text.is_empty() {
					return Some(text.to_string());
				}
			}
			_ => {}
		}
	}
	None
}

fn token_text<'a>(tok: &Token, source: &'a str) -> &'a str {
	let start = tok.span.offset as usize;
	let end = start + tok.span.len as usize;
	if end <= source.len() {
		&source[start..end]
	} else {
		""
	}
}

/// A table reference found in a DML statement (SELECT FROM, UPDATE, CREATE, etc.)
#[derive(Debug, Clone)]
pub struct TableReference {
	pub name: String,
	pub line: u32,
	pub col: u32,
	pub len: u32,
}

/// Extract all table references from DML statements in the source.
///
/// Finds tables referenced via FROM, UPDATE, CREATE, DELETE, UPSERT, INSERT INTO.
/// Skips tables in DEFINE statements (those are definitions, not references).
pub fn extract_table_references(source: &str) -> Vec<TableReference> {
	let bytes = source.as_bytes();
	if bytes.is_empty() || bytes.len() > u32::MAX as usize {
		return Vec::new();
	}

	let tokens: Vec<Token> = match std::panic::catch_unwind(|| Lexer::new(bytes).collect()) {
		Ok(t) => t,
		Err(_) => return Vec::new(),
	};

	let is_keyword = |idx: usize| -> bool { matches!(tokens[idx].kind, TokenKind::Keyword(_)) };

	let text_of = |idx: usize| -> &str { token_text(&tokens[idx], source) };

	let mut refs = Vec::new();
	let mut in_define = false;

	for i in 0..tokens.len() {
		let tok = &tokens[i];

		if tok.kind == TokenKind::SemiColon {
			in_define = false;
			continue;
		}

		if is_keyword(i) && text_of(i).eq_ignore_ascii_case("DEFINE") {
			in_define = true;
			continue;
		}

		if in_define {
			continue;
		}

		if !is_keyword(i) {
			continue;
		}

		let upper = text_of(i).to_uppercase();
		let table_idx = match upper.as_str() {
			"FROM" | "UPDATE" | "CREATE" | "DELETE" | "UPSERT" => {
				skip_modifiers(i, &tokens, source)
			}
			"INTO" => {
				if i > 0 && is_keyword(i - 1) && text_of(i - 1).eq_ignore_ascii_case("INSERT") {
					skip_modifiers(i, &tokens, source)
				} else {
					None
				}
			}
			_ => None,
		};

		if let Some(idx) = table_idx
			&& idx < tokens.len()
		{
			let tok = &tokens[idx];
			if matches!(tok.kind, TokenKind::Identifier | TokenKind::Keyword(_)) {
				let name = text_of(idx);
				let name = name.strip_prefix('`').unwrap_or(name);
				let name = name.strip_suffix('`').unwrap_or(name);
				if !name.is_empty() && !is_surql_builtin_table(name) {
					let offset = tok.span.offset as usize;
					let before = &source[..offset];
					let line = before.matches('\n').count() as u32;
					let col = before
						.rfind('\n')
						.map(|nl| offset - nl - 1)
						.unwrap_or(offset) as u32;
					refs.push(TableReference {
						name: name.to_string(),
						line,
						col,
						len: tok.span.len,
					});
				}
			}
		}
	}

	refs
}

/// Skip ONLY / TABLE modifiers after a keyword, returning the index of the table name token.
fn skip_modifiers(kw_idx: usize, tokens: &[Token], source: &str) -> Option<usize> {
	let mut next = kw_idx + 1;
	while next < tokens.len() && matches!(tokens[next].kind, TokenKind::Keyword(_)) {
		let text = token_text(&tokens[next], source);
		if text.eq_ignore_ascii_case("ONLY") || text.eq_ignore_ascii_case("TABLE") {
			next += 1;
		} else {
			break;
		}
	}
	if next < tokens.len() {
		Some(next)
	} else {
		None
	}
}

fn is_surql_builtin_table(name: &str) -> bool {
	let lower = name.to_lowercase();
	// SurrealDB system/special identifiers that aren't user tables
	matches!(
		lower.as_str(),
		"none" | "null" | "true" | "false" | "only" | "type" | "if" | "else" | "then" | "end"
	)
}
