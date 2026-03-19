//! Completion provider — keywords + schema-aware suggestions.

use surql_parser::SchemaGraph;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, Position};

use crate::keywords::KEYWORDS;

/// Context detected at cursor position.
#[derive(Debug, PartialEq)]
pub(crate) enum Context {
	/// After FROM, INTO, ON — suggest table names.
	TableName,
	/// After `fn::` — suggest function names.
	FunctionName,
	/// After `$` — suggest param names.
	ParamName,
	/// General context — suggest keywords.
	General,
}

/// Generate completions based on cursor position and schema.
pub fn complete(
	source: &str,
	position: Position,
	schema: Option<&SchemaGraph>,
) -> Vec<CompletionItem> {
	let mut items = Vec::new();
	let context = detect_context(source, position);

	match context {
		Context::TableName => {
			if let Some(sg) = schema {
				for name in sg.table_names() {
					items.push(CompletionItem {
						label: name.to_string(),
						kind: Some(CompletionItemKind::CLASS),
						detail: Some("table".into()),
						..Default::default()
					});
				}
			}
			// Also suggest keywords (user might be typing a keyword, not a table)
			items.extend(keyword_completions());
		}
		Context::FunctionName => {
			if let Some(sg) = schema {
				for name in sg.function_names() {
					let func = sg.function(name);
					let detail = func.map(|f| {
						let args = f
							.args
							.iter()
							.map(|(n, t)| format!("{n}: {t}"))
							.collect::<Vec<_>>()
							.join(", ");
						let ret = f
							.returns
							.as_ref()
							.map(|r| format!(" -> {r}"))
							.unwrap_or_default();
						format!("({args}){ret}")
					});
					items.push(CompletionItem {
						label: format!("fn::{name}"),
						kind: Some(CompletionItemKind::FUNCTION),
						detail,
						..Default::default()
					});
				}
			}
		}
		Context::ParamName => {
			if let Some(sg) = schema {
				for name in sg.param_names() {
					items.push(CompletionItem {
						label: format!("${name}"),
						kind: Some(CompletionItemKind::VARIABLE),
						..Default::default()
					});
				}
			}
		}
		Context::General => {
			items.extend(keyword_completions());
			if let Some(sg) = schema {
				for name in sg.table_names() {
					items.push(CompletionItem {
						label: name.to_string(),
						kind: Some(CompletionItemKind::CLASS),
						detail: Some("table".into()),
						..Default::default()
					});
				}
				for name in sg.function_names() {
					items.push(CompletionItem {
						label: format!("fn::{name}"),
						kind: Some(CompletionItemKind::FUNCTION),
						..Default::default()
					});
				}
			}
		}
	}

	items
}

fn keyword_completions() -> Vec<CompletionItem> {
	KEYWORDS
		.iter()
		.map(|kw| CompletionItem {
			label: kw.to_string(),
			kind: Some(CompletionItemKind::KEYWORD),
			..Default::default()
		})
		.collect()
}

/// Detect the completion context by scanning backwards from cursor.
pub(crate) fn detect_context(source: &str, position: Position) -> Context {
	let line_idx = position.line as usize;
	let col = position.character as usize;

	let line = source.lines().nth(line_idx).unwrap_or("");
	let before_cursor = if col <= line.len() {
		&line[..col]
	} else {
		line
	};

	let trimmed = before_cursor.trim_end();

	// After $ — param completion
	if trimmed.ends_with('$') || before_cursor.ends_with('$') {
		return Context::ParamName;
	}

	// After fn:: — function completion
	if trimmed.ends_with("fn::") || trimmed.ends_with("fn:") {
		return Context::FunctionName;
	}

	// After FROM, INTO, ON — table completion
	// Check both the raw before_cursor (may end with space) and trimmed version
	let upper_raw = before_cursor.to_uppercase();
	let upper_trimmed = trimmed.to_uppercase();
	for keyword in &["FROM", "INTO", "ON", "TABLE"] {
		// "FROM " with cursor after space, or "FROM\t"
		if upper_raw.ends_with(&format!("{keyword} "))
			|| upper_raw.ends_with(&format!("{keyword}\t"))
		{
			return Context::TableName;
		}
		// Just typed "FROM" with cursor right after (no space yet)
		if upper_trimmed.ends_with(keyword) {
			// Only if it's a word boundary (not part of a longer word)
			let prefix = &upper_trimmed[..upper_trimmed.len() - keyword.len()];
			if prefix.is_empty()
				|| prefix.ends_with(' ')
				|| prefix.ends_with('\t')
				|| prefix.ends_with(';')
			{
				return Context::TableName;
			}
		}
	}

	Context::General
}
