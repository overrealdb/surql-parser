//! Document formatting for SurrealQL.
//!
//! Two modes:
//! - `canonical-format` feature: upstream SurrealDB formatter (destructive — removes
//!   comments, changes structure). Off by default.
//! - Default: keyword uppercaser — preserves comments, blank lines, indentation.
//!   Only normalizes SQL keyword casing (select → SELECT, define → DEFINE).

use tower_lsp::lsp_types::{Position, Range, TextEdit};

#[cfg(test)]
pub fn format_document_for_test(source: &str) -> Option<Vec<TextEdit>> {
	format_document(source, true)
}

pub fn format_document(source: &str, enabled: bool) -> Option<Vec<TextEdit>> {
	if !enabled {
		return None;
	}

	if cfg!(feature = "canonical-format") {
		canonical_format(source)
	} else {
		keyword_format(source)
	}
}

/// Upstream SurrealDB canonical formatter (destructive).
fn canonical_format(source: &str) -> Option<Vec<TextEdit>> {
	let ast = surql_parser::parse(source).ok()?;
	let formatted = surql_parser::format(&ast);
	if formatted == source {
		return None;
	}
	let last_line_idx = source.lines().count().saturating_sub(1) as u32;
	let last_line_chars = source
		.lines()
		.last()
		.map(|l| l.chars().count())
		.unwrap_or(0) as u32;
	Some(vec![TextEdit {
		range: Range {
			start: Position {
				line: 0,
				character: 0,
			},
			end: Position {
				line: last_line_idx,
				character: last_line_chars,
			},
		},
		new_text: formatted,
	}])
}

/// Keyword-only formatter: uppercases SQL keywords while preserving everything else.
fn keyword_format(source: &str) -> Option<Vec<TextEdit>> {
	use surql_parser::upstream::syn::lexer::Lexer;
	use surql_parser::upstream::syn::token::TokenKind;

	let bytes = source.as_bytes();
	if bytes.is_empty() || bytes.len() > u32::MAX as usize {
		return None;
	}

	let tokens: Vec<_> = match std::panic::catch_unwind(|| Lexer::new(bytes).collect()) {
		Ok(t) => t,
		Err(_) => return None,
	};

	let mut edits = Vec::new();

	for token in &tokens {
		if !matches!(token.kind, TokenKind::Keyword(_)) {
			continue;
		}

		let start = token.span.offset as usize;
		let end = start + token.span.len as usize;
		if end > source.len() {
			continue;
		}

		let original = &source[start..end];
		let upper = original.to_uppercase();

		if !is_formattable_keyword(&upper) {
			continue;
		}
		if original == upper {
			continue;
		}

		let before = &source[..start];
		let line = before.matches('\n').count() as u32;
		let col = before.rfind('\n').map(|nl| start - nl - 1).unwrap_or(start) as u32;

		edits.push(TextEdit {
			range: Range {
				start: Position {
					line,
					character: col,
				},
				end: Position {
					line,
					character: col + original.len() as u32,
				},
			},
			new_text: upper,
		});
	}

	if edits.is_empty() { None } else { Some(edits) }
}

fn is_formattable_keyword(upper: &str) -> bool {
	matches!(
		upper,
		"SELECT"
			| "FROM" | "WHERE"
			| "AND" | "OR"
			| "NOT" | "IN"
			| "ORDER" | "BY"
			| "GROUP" | "LIMIT"
			| "OFFSET"
			| "FETCH" | "CREATE"
			| "UPDATE"
			| "DELETE"
			| "INSERT"
			| "INTO" | "SET"
			| "UPSERT"
			| "MERGE" | "CONTENT"
			| "RETURN"
			| "DEFINE"
			| "REMOVE"
			| "TABLE" | "FIELD"
			| "INDEX" | "EVENT"
			| "FUNCTION"
			| "PARAM" | "NAMESPACE"
			| "DATABASE"
			| "ANALYZER"
			| "ACCESS"
			| "SCHEMAFULL"
			| "SCHEMALESS"
			| "TYPE" | "DEFAULT"
			| "READONLY"
			| "FLEXIBLE"
			| "UNIQUE"
			| "FIELDS"
			| "ON" | "WHEN"
			| "THEN" | "ELSE"
			| "END" | "IF"
			| "FOR" | "LET"
			| "BEGIN" | "COMMIT"
			| "CANCEL"
			| "TRANSACTION"
			| "USE" | "NS"
			| "DB" | "AS"
			| "IS" | "LIKE"
			| "CONTAINS"
			| "CONTAINSALL"
			| "CONTAINSANY"
			| "CONTAINSNONE"
			| "INSIDE"
			| "OUTSIDE"
			| "INTERSECTS"
			| "ALLINSIDE"
			| "ANYINSIDE"
			| "NONEINSIDE"
			| "ASC" | "DESC"
			| "COLLATE"
			| "NUMERIC"
			| "COMMENT"
			| "PERMISSIONS"
			| "FULL" | "NONE"
			| "RELATE"
			| "ONLY" | "VALUE"
			| "VALUES"
			| "OVERWRITE"
			| "EXISTS"
			| "ASSERT"
			| "ENFORCED"
			| "DROP" | "CHANGEFEED"
			| "INCLUDE"
			| "ORIGINAL"
			| "LIVE" | "DIFF"
			| "KILL" | "SHOW"
			| "INFO" | "SLEEP"
			| "THROW" | "BREAK"
			| "CONTINUE"
			| "PARALLEL"
			| "TIMEOUT"
			| "EXPLAIN"
			| "SPLIT" | "AT"
			| "TOKENIZERS"
			| "FILTERS"
			| "WITH" | "NOINDEX"
			| "UNIQ" | "SEARCH"
	)
}
