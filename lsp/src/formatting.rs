//! Document formatting via surql-parser.
//!
//! Disabled by default — upstream SurrealDB formatter destroys comments,
//! blank lines, indentation, and adds verbose defaults (PERMISSIONS FULL,
//! TYPE NORMAL). Also corrupts DEFINE FUNCTION bodies.
//!
//! Enable with `--features canonical-format` or set `"surql.format": true`
//! in workspace settings.

use tower_lsp::lsp_types::{Position, Range, TextEdit};

#[cfg(test)]
pub fn format_document_for_test(source: &str) -> Option<Vec<TextEdit>> {
	format_document(source, true)
}

pub fn format_document(source: &str, enabled: bool) -> Option<Vec<TextEdit>> {
	if !enabled {
		return None;
	}
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
