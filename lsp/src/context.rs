//! Determine the table context at a given cursor position in SurrealQL source.
//!
//! Scans backwards through tokens from the cursor to find the most recent
//! statement keyword (SELECT..FROM, UPDATE, CREATE, etc.) and extracts the
//! table name that follows it.
//!
//! Also provides schema-aware reference search: field references scoped to
//! their table, table references via lexer-based pattern matching, and
//! function call references.

use surql_parser::upstream::syn::lexer::Lexer;
use surql_parser::upstream::syn::token::{Token, TokenKind};
use tower_lsp::lsp_types::{Position, Url};

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
		Err(e) => {
			tracing::error!("Lexer panicked in table_context_at_position: {e:?}");
			return None;
		}
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
		Err(e) => {
			tracing::error!("Lexer panicked in extract_table_references: {e:?}");
			return Vec::new();
		}
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

/// Extract ALL occurrences of a table name in source, including definitions and type positions.
///
/// Covers:
/// - `DEFINE TABLE <name>` (the table name token)
/// - DML keywords: `FROM <name>`, `UPDATE <name>`, `CREATE <name>`, `DELETE <name>`,
///   `UPSERT <name>`, `INSERT INTO <name>`
/// - `DEFINE FIELD/INDEX/EVENT ... ON [TABLE] <name>`
/// - `record<name>` (type annotation)
/// - `<name>:record_id` (record ID prefix, before `:`)
pub fn extract_all_table_occurrences(source: &str, target: &str) -> Vec<TableReference> {
	let bytes = source.as_bytes();
	if bytes.is_empty() || bytes.len() > u32::MAX as usize {
		return Vec::new();
	}

	let tokens: Vec<Token> = match std::panic::catch_unwind(|| Lexer::new(bytes).collect()) {
		Ok(t) => t,
		Err(e) => {
			tracing::error!("Lexer panicked in extract_all_table_occurrences: {e:?}");
			return Vec::new();
		}
	};

	let is_keyword = |idx: usize| -> bool { matches!(tokens[idx].kind, TokenKind::Keyword(_)) };

	let text_of = |idx: usize| -> &str { token_text(&tokens[idx], source) };

	let matches_target = |idx: usize| -> bool {
		if !matches!(
			tokens[idx].kind,
			TokenKind::Identifier | TokenKind::Keyword(_)
		) {
			return false;
		}
		let t = text_of(idx);
		let t = t.strip_prefix('`').unwrap_or(t);
		let t = t.strip_suffix('`').unwrap_or(t);
		t.eq_ignore_ascii_case(target)
	};

	let to_ref = |idx: usize| -> TableReference {
		let tok = &tokens[idx];
		let offset = tok.span.offset as usize;
		let before = &source[..offset];
		let line = before.matches('\n').count() as u32;
		let col = before
			.rfind('\n')
			.map(|nl| offset - nl - 1)
			.unwrap_or(offset) as u32;
		let name = text_of(idx);
		let name = name.strip_prefix('`').unwrap_or(name);
		let name = name.strip_suffix('`').unwrap_or(name);
		TableReference {
			name: name.to_string(),
			line,
			col,
			len: tok.span.len,
		}
	};

	let mut refs = Vec::new();

	for i in 0..tokens.len() {
		if !is_keyword(i) && !matches!(tokens[i].kind, TokenKind::Identifier) {
			continue;
		}

		let upper = text_of(i).to_uppercase();

		match upper.as_str() {
			// DEFINE TABLE <name>
			"TABLE" => {
				if i > 0 && is_keyword(i - 1) && text_of(i - 1).eq_ignore_ascii_case("DEFINE") {
					let next = i + 1;
					if next < tokens.len() && matches_target(next) {
						refs.push(to_ref(next));
					}
				}
			}
			// DML: FROM, UPDATE, CREATE, DELETE, UPSERT
			"FROM" | "UPDATE" | "CREATE" | "DELETE" | "UPSERT" => {
				if let Some(idx) = skip_modifiers(i, &tokens, source)
					&& idx < tokens.len()
					&& matches_target(idx)
				{
					refs.push(to_ref(idx));
				}
			}
			// INSERT INTO <name>
			"INTO" => {
				if i > 0
					&& is_keyword(i - 1)
					&& text_of(i - 1).eq_ignore_ascii_case("INSERT")
					&& let Some(idx) = skip_modifiers(i, &tokens, source)
					&& idx < tokens.len()
					&& matches_target(idx)
				{
					refs.push(to_ref(idx));
				}
			}
			// DEFINE FIELD/INDEX/EVENT ... ON [TABLE] <name>
			"ON" => {
				let has_define_context = (0..i).rev().any(|j| {
					is_keyword(j) && {
						let kw = text_of(j).to_uppercase();
						matches!(kw.as_str(), "FIELD" | "INDEX" | "EVENT")
					}
				});
				if has_define_context
					&& let Some(idx) = skip_modifiers(i, &tokens, source)
					&& idx < tokens.len()
					&& matches_target(idx)
				{
					refs.push(to_ref(idx));
				}
			}
			_ => {}
		}

		// record<name> and name:id patterns
		if (tokens[i].kind == TokenKind::Identifier || is_keyword(i)) && matches_target(i) && i >= 2
		{
			// record<name> pattern: RECORD < name >
			if tokens[i - 1].kind == TokenKind::LeftChefron
				&& matches!(
					tokens[i - 2].kind,
					TokenKind::Identifier | TokenKind::Keyword(_)
				) && text_of(i - 2).eq_ignore_ascii_case("record")
			{
				refs.push(to_ref(i));
			}

			// name:id pattern — table name followed by Colon
			if i + 1 < tokens.len()
				&& tokens[i + 1].kind == TokenKind::Colon
				&& (i == 0 || tokens[i - 1].kind != TokenKind::Dot)
			{
				refs.push(to_ref(i));
			}
		}
	}

	// Deduplicate by (line, col) since a token may match multiple patterns
	refs.sort_by_key(|r| (r.line, r.col));
	refs.dedup_by_key(|r| (r.line, r.col));
	refs
}

/// What kind of symbol the cursor is on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
	/// A table name (e.g. `user` in `FROM user` or `DEFINE TABLE user`).
	Table(String),
	/// A field name scoped to a table (e.g. `name` on table `user`).
	Field { table: String, field: String },
	/// A function name (e.g. `fn::greet`).
	Function(String),
	/// Unknown — fall back to plain word search.
	Unknown(String),
}

/// Classify the symbol at the cursor position.
///
/// Lexes the full source, finds the token containing the cursor, then walks
/// backwards through the token stream to determine if it is a table, field,
/// or function reference.
pub fn classify_symbol_at_position(source: &str, position: Position) -> Option<SymbolKind> {
	let cursor_byte = position_to_byte_offset(source, position);

	// Extract the word under cursor (for fn:: prefix detection and fallback)
	let line = source.lines().nth(position.line as usize)?;
	let col = position.character as usize;
	let line_bytes = line.as_bytes();

	let start = (0..col)
		.rev()
		.take_while(|&i| {
			i < line_bytes.len()
				&& (line_bytes[i].is_ascii_alphanumeric()
					|| line_bytes[i] == b'_'
					|| line_bytes[i] == b':')
		})
		.last()
		.unwrap_or(col);

	let end = (col..line_bytes.len())
		.take_while(|&i| {
			line_bytes[i].is_ascii_alphanumeric() || line_bytes[i] == b'_' || line_bytes[i] == b':'
		})
		.last()
		.map(|i| i + 1)
		.unwrap_or(col);

	if start >= end || end > line.len() {
		return None;
	}

	let word = &line[start..end];
	if word.is_empty() {
		return None;
	}

	if word.starts_with("fn::") {
		return Some(SymbolKind::Function(word.to_string()));
	}

	let src_bytes = source.as_bytes();
	if src_bytes.is_empty() || src_bytes.len() > u32::MAX as usize {
		return Some(SymbolKind::Unknown(word.to_string()));
	}

	let tokens: Vec<Token> = match std::panic::catch_unwind(|| Lexer::new(src_bytes).collect()) {
		Ok(t) => t,
		Err(e) => {
			tracing::error!("Lexer panicked in classify_symbol: {e:?}");
			return Some(SymbolKind::Unknown(word.to_string()));
		}
	};

	if tokens.is_empty() {
		return Some(SymbolKind::Unknown(word.to_string()));
	}

	let text_of = |idx: usize| -> &str { token_text(&tokens[idx], source) };

	let is_kw = |idx: usize| -> bool { matches!(tokens[idx].kind, TokenKind::Keyword(_)) };

	// Find the token at cursor_byte
	let cursor_tok_idx = tokens.iter().position(|tok| {
		let tok_start = tok.span.offset as usize;
		let tok_end = tok_start + tok.span.len as usize;
		cursor_byte >= tok_start && cursor_byte <= tok_end
	});

	let cursor_idx = match cursor_tok_idx {
		Some(idx) => idx,
		None => return Some(SymbolKind::Unknown(word.to_string())),
	};

	// Find the statement start (scan backwards to semicolon or beginning)
	let stmt_start = (0..cursor_idx)
		.rev()
		.find(|&i| tokens[i].kind == TokenKind::SemiColon)
		.map(|i| i + 1)
		.unwrap_or(0);

	// Walk backwards from cursor to find the relevant keyword context
	for i in (stmt_start..=cursor_idx).rev() {
		if !is_kw(i) {
			continue;
		}

		let upper = text_of(i).to_uppercase();
		match upper.as_str() {
			"FIELD" => {
				if i > 0 && is_kw(i - 1) && text_of(i - 1).eq_ignore_ascii_case("DEFINE") {
					if let Some(field_idx) = next_ident_index(i, &tokens) {
						let field_name = strip_backticks(text_of(field_idx));
						if (field_idx == cursor_idx || field_name.eq_ignore_ascii_case(word))
							&& let Some(table) = find_on_table_after(field_idx, &tokens, source)
						{
							return Some(SymbolKind::Field {
								table,
								field: field_name.to_string(),
							});
						}
					}
					if let Some(on_idx) = find_keyword_after(i, "ON", &tokens, source)
						&& let Some(table_idx) = skip_table_kw(on_idx, &tokens, source)
						&& table_idx == cursor_idx
					{
						let table_name = strip_backticks(text_of(table_idx)).to_string();
						return Some(SymbolKind::Table(table_name));
					}
				}
			}
			"TABLE" => {
				if i > 0 && is_kw(i - 1) && text_of(i - 1).eq_ignore_ascii_case("DEFINE") {
					return Some(SymbolKind::Table(word.to_string()));
				}
			}
			"FROM" | "UPDATE" | "CREATE" | "DELETE" | "UPSERT" => {
				if let Some(table_idx) = skip_only_modifier(i, &tokens, source)
					&& table_idx == cursor_idx
				{
					let t = strip_backticks(text_of(table_idx));
					return Some(SymbolKind::Table(t.to_string()));
				}
				if let Some(table) = extract_table_name(i, &tokens, source)
					&& cursor_idx > skip_only_modifier(i, &tokens, source).unwrap_or(i + 1)
				{
					return Some(SymbolKind::Field {
						table,
						field: word.to_string(),
					});
				}
			}
			"INTO" => {
				if i > 0 && is_kw(i - 1) && text_of(i - 1).eq_ignore_ascii_case("INSERT") {
					if let Some(table_idx) = skip_only_modifier(i, &tokens, source)
						&& table_idx == cursor_idx
					{
						let t = strip_backticks(text_of(table_idx));
						return Some(SymbolKind::Table(t.to_string()));
					}
					if let Some(table) = extract_table_name(i, &tokens, source)
						&& cursor_idx > skip_only_modifier(i, &tokens, source).unwrap_or(i + 1)
					{
						return Some(SymbolKind::Field {
							table,
							field: word.to_string(),
						});
					}
				}
			}
			"ON" => {
				if let Some(table_idx) = skip_table_kw(i, &tokens, source)
					&& table_idx == cursor_idx
				{
					let t = strip_backticks(text_of(table_idx)).to_string();
					return Some(SymbolKind::Table(t));
				}
			}
			"FUNCTION" => {
				if i > 0 && is_kw(i - 1) && text_of(i - 1).eq_ignore_ascii_case("DEFINE") {
					return Some(SymbolKind::Function(word.to_string()));
				}
			}
			_ => {}
		}
	}

	Some(SymbolKind::Unknown(word.to_string()))
}

fn strip_backticks(s: &str) -> &str {
	let s = s.strip_prefix('`').unwrap_or(s);
	s.strip_suffix('`').unwrap_or(s)
}

fn next_ident_index(after: usize, tokens: &[Token]) -> Option<usize> {
	let next = after + 1;
	if next < tokens.len()
		&& matches!(
			tokens[next].kind,
			TokenKind::Identifier | TokenKind::Keyword(_)
		) {
		Some(next)
	} else {
		None
	}
}

fn skip_only_modifier(kw_idx: usize, tokens: &[Token], source: &str) -> Option<usize> {
	let mut next = kw_idx + 1;
	if next < tokens.len()
		&& matches!(tokens[next].kind, TokenKind::Keyword(_))
		&& token_text(&tokens[next], source).eq_ignore_ascii_case("ONLY")
	{
		next += 1;
	}
	if next < tokens.len()
		&& matches!(
			tokens[next].kind,
			TokenKind::Identifier | TokenKind::Keyword(_)
		) {
		Some(next)
	} else {
		None
	}
}

fn skip_table_kw(on_idx: usize, tokens: &[Token], source: &str) -> Option<usize> {
	let mut next = on_idx + 1;
	if next < tokens.len()
		&& matches!(tokens[next].kind, TokenKind::Keyword(_))
		&& token_text(&tokens[next], source).eq_ignore_ascii_case("TABLE")
	{
		next += 1;
	}
	if next < tokens.len()
		&& matches!(
			tokens[next].kind,
			TokenKind::Identifier | TokenKind::Keyword(_)
		) {
		Some(next)
	} else {
		None
	}
}

fn find_keyword_after(
	start: usize,
	keyword: &str,
	tokens: &[Token],
	source: &str,
) -> Option<usize> {
	for (i, tok) in tokens.iter().enumerate().skip(start + 1) {
		if tok.kind == TokenKind::SemiColon {
			return None;
		}
		if matches!(tok.kind, TokenKind::Keyword(_))
			&& token_text(tok, source).eq_ignore_ascii_case(keyword)
		{
			return Some(i);
		}
	}
	None
}

fn find_on_table_after(field_idx: usize, tokens: &[Token], source: &str) -> Option<String> {
	if let Some(on_idx) = find_keyword_after(field_idx, "ON", tokens, source)
		&& let Some(table_idx) = skip_table_kw(on_idx, tokens, source)
	{
		let text = token_text(&tokens[table_idx], source);
		let text = strip_backticks(text);
		if !text.is_empty() {
			return Some(text.to_string());
		}
	}
	None
}

/// A reference location within source text (line/col/len, no URI).
#[derive(Debug, Clone)]
pub struct SourceReference {
	pub line: u32,
	pub col: u32,
	pub len: u32,
}

fn offset_to_line_col(source: &str, offset: usize) -> (u32, u32) {
	let before = &source[..offset.min(source.len())];
	let line = before.matches('\n').count() as u32;
	let col = before
		.rfind('\n')
		.map(|nl| offset - nl - 1)
		.unwrap_or(offset) as u32;
	(line, col)
}

/// Find all references to a field within a source document.
///
/// Matches:
/// - `table.field` (dot access)
/// - `field` inside statements that target the given table (after FROM/UPDATE/etc)
/// - `DEFINE FIELD field ON [TABLE] table`
pub fn find_field_references(source: &str, table: &str, field: &str) -> Vec<SourceReference> {
	let bytes = source.as_bytes();
	if bytes.is_empty() || bytes.len() > u32::MAX as usize {
		return Vec::new();
	}

	let tokens: Vec<Token> = match std::panic::catch_unwind(|| Lexer::new(bytes).collect()) {
		Ok(t) => t,
		Err(e) => {
			tracing::error!("Lexer panicked in find_field_references: {e:?}");
			return Vec::new();
		}
	};

	let text_of = |idx: usize| -> &str { token_text(&tokens[idx], source) };

	let is_ident_like = |idx: usize| -> bool {
		matches!(
			tokens[idx].kind,
			TokenKind::Identifier | TokenKind::Keyword(_)
		)
	};

	let matches_field = |idx: usize| -> bool {
		if !is_ident_like(idx) {
			return false;
		}
		strip_backticks(text_of(idx)).eq_ignore_ascii_case(field)
	};

	let matches_table_name = |idx: usize| -> bool {
		if !is_ident_like(idx) {
			return false;
		}
		strip_backticks(text_of(idx)).eq_ignore_ascii_case(table)
	};

	let mut refs = Vec::new();
	let mut seen_offsets = std::collections::HashSet::new();

	let push_ref = |refs: &mut Vec<SourceReference>,
	                seen: &mut std::collections::HashSet<u32>,
	                tok: &Token| {
		if seen.insert(tok.span.offset) {
			let (line, col) = offset_to_line_col(source, tok.span.offset as usize);
			refs.push(SourceReference {
				line,
				col,
				len: tok.span.len,
			});
		}
	};

	// Pass 1: Find `table.field` dot-access patterns
	for i in 0..tokens.len() {
		if matches_table_name(i)
			&& i + 2 < tokens.len()
			&& tokens[i + 1].kind == TokenKind::Dot
			&& matches_field(i + 2)
		{
			push_ref(&mut refs, &mut seen_offsets, &tokens[i + 2]);
		}
	}

	// Pass 2: Find bare `field` in statement context targeting this table
	let mut i = 0;
	while i < tokens.len() {
		let tok = &tokens[i];

		if tok.kind == TokenKind::SemiColon {
			i += 1;
			continue;
		}

		if matches!(tok.kind, TokenKind::Keyword(_)) {
			let upper = text_of(i).to_uppercase();
			match upper.as_str() {
				"DEFINE" => {
					// DEFINE FIELD <field> ON [TABLE] <table>
					if i + 1 < tokens.len()
						&& matches!(tokens[i + 1].kind, TokenKind::Keyword(_))
						&& text_of(i + 1).eq_ignore_ascii_case("FIELD")
						&& let Some(field_idx) = next_ident_index(i + 1, &tokens)
						&& matches_field(field_idx)
						&& let Some(on_table) = find_on_table_after(field_idx, &tokens, source)
						&& on_table.eq_ignore_ascii_case(table)
					{
						push_ref(&mut refs, &mut seen_offsets, &tokens[field_idx]);
					}
					i += 1;
					continue;
				}
				"FROM" | "UPDATE" | "CREATE" | "DELETE" | "UPSERT" => {
					if let Some(ref t) = extract_table_name(i, &tokens, source)
						&& t.eq_ignore_ascii_case(table)
					{
						let stmt_end = find_statement_end(i, &tokens);
						scan_bare_field_refs(
							i,
							stmt_end,
							field,
							&tokens,
							source,
							&mut refs,
							&mut seen_offsets,
						);
					}
				}
				"INTO" => {
					if i > 0
						&& matches!(tokens[i - 1].kind, TokenKind::Keyword(_))
						&& text_of(i - 1).eq_ignore_ascii_case("INSERT")
						&& let Some(ref t) = extract_table_name(i, &tokens, source)
						&& t.eq_ignore_ascii_case(table)
					{
						let stmt_end = find_statement_end(i, &tokens);
						scan_bare_field_refs(
							i,
							stmt_end,
							field,
							&tokens,
							source,
							&mut refs,
							&mut seen_offsets,
						);
					}
				}
				_ => {}
			}
		}

		i += 1;
	}

	refs
}

fn find_statement_end(from: usize, tokens: &[Token]) -> usize {
	for (i, tok) in tokens.iter().enumerate().skip(from) {
		if tok.kind == TokenKind::SemiColon || tok.kind == TokenKind::Eof {
			return i;
		}
	}
	tokens.len()
}

fn scan_bare_field_refs(
	from: usize,
	to: usize,
	field: &str,
	tokens: &[Token],
	source: &str,
	refs: &mut Vec<SourceReference>,
	seen: &mut std::collections::HashSet<u32>,
) {
	let table_idx = skip_modifiers(from, tokens, source).unwrap_or(from + 1);
	let scan_start = table_idx + 1;

	for i in scan_start..to {
		if matches!(
			tokens[i].kind,
			TokenKind::Identifier | TokenKind::Keyword(_)
		) {
			let t = strip_backticks(token_text(&tokens[i], source));
			if t.eq_ignore_ascii_case(field) {
				if i > 0 && tokens[i - 1].kind == TokenKind::Dot {
					continue;
				}
				if is_dml_keyword(t) {
					continue;
				}
				if seen.insert(tokens[i].span.offset) {
					let (line, col) = offset_to_line_col(source, tokens[i].span.offset as usize);
					refs.push(SourceReference {
						line,
						col,
						len: tokens[i].span.len,
					});
				}
			}
		}
	}
}

fn is_dml_keyword(word: &str) -> bool {
	let upper = word.to_uppercase();
	matches!(
		upper.as_str(),
		"SELECT"
			| "FROM" | "WHERE"
			| "SET" | "CONTENT"
			| "MERGE" | "RETURN"
			| "LIMIT" | "ORDER"
			| "BY" | "GROUP"
			| "FETCH" | "TIMEOUT"
			| "PARALLEL"
			| "EXPLAIN"
			| "DEFINE"
			| "TABLE" | "FIELD"
			| "INDEX" | "EVENT"
			| "ON" | "TYPE"
			| "VALUE" | "DEFAULT"
			| "ASSERT"
			| "PERMISSIONS"
			| "CREATE"
			| "UPDATE"
			| "DELETE"
			| "INSERT"
			| "INTO" | "UPSERT"
			| "AND" | "OR"
			| "NOT" | "IN"
			| "IS" | "LET"
			| "IF" | "ELSE"
			| "THEN" | "END"
			| "FOR" | "BREAK"
			| "CONTINUE"
			| "BEGIN" | "CANCEL"
			| "COMMIT"
			| "ONLY" | "SCHEMAFULL"
			| "SCHEMALESS"
			| "AS" | "UNIQUE"
			| "WHEN" | "LIVE"
			| "SPLIT" | "WITH"
			| "ASC" | "DESC"
			| "COLLATE"
			| "NUMERIC"
			| "TRUE" | "FALSE"
			| "NONE" | "NULL"
			| "FLEXIBLE"
			| "READONLY"
			| "OVERWRITE"
	)
}

/// Find all references to a `fn::name` function in source.
///
/// Uses word-boundary matching: the fn::name must not be preceded by an
/// alphanumeric or underscore, and must be followed by `(`, whitespace,
/// `;`, `,`, `)`, or end-of-line.
pub fn find_function_references_in(source: &str, fn_name: &str) -> Vec<SourceReference> {
	let fn_bytes = fn_name.as_bytes();
	let mut refs = Vec::new();

	for (line_num, line) in source.lines().enumerate() {
		let line_bytes = line.as_bytes();
		let mut col = 0;
		while col + fn_bytes.len() <= line_bytes.len() {
			if line_bytes[col..].starts_with(fn_bytes) {
				let before_ok = col == 0
					|| !(line_bytes[col - 1].is_ascii_alphanumeric()
						|| line_bytes[col - 1] == b'_');
				let after_pos = col + fn_bytes.len();
				let after_ok = after_pos >= line_bytes.len()
					|| matches!(
						line_bytes[after_pos],
						b'(' | b')' | b' ' | b'\t' | b';' | b',' | b'\n' | b'\r'
					);

				if before_ok && after_ok {
					refs.push(SourceReference {
						line: line_num as u32,
						col: col as u32,
						len: fn_bytes.len() as u32,
					});
				}
			}
			col += 1;
		}
	}

	refs
}

/// Collect all .surql file paths under a workspace root and read their contents.
pub fn collect_workspace_surql_sources(root: &std::path::Path) -> Vec<(Url, String)> {
	let mut files = Vec::new();
	surql_parser::collect_surql_files(root, &mut files);
	let mut sources = Vec::new();
	for path in files {
		if let Ok(content) = std::fs::read_to_string(&path)
			&& let Ok(uri) = Url::from_file_path(&path)
		{
			sources.push((uri, content));
		}
	}
	sources
}
