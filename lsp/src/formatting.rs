//! Document formatting via surql-parser.

use tower_lsp::lsp_types::{Position, Range, TextEdit};

/// Format a SurrealQL document, returning a single TextEdit replacing the entire content.
pub fn format_document(source: &str) -> Option<Vec<TextEdit>> {
	let ast = surql_parser::parse(source).ok()?;
	let formatted = surql_parser::format(&ast);
	if formatted == source {
		return None;
	}
	// LSP positions are 0-indexed. End position = after last character.
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
