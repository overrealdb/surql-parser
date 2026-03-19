//! Completion provider — keywords + schema-aware suggestions.

use surql_parser::SchemaGraph;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat, Position};

use crate::keywords::KEYWORDS;

/// Context detected at cursor position.
#[derive(Debug, PartialEq)]
pub(crate) enum Context {
	/// After FROM, INTO, ON — suggest table names.
	TableName,
	/// After `table.` — suggest fields of that table.
	FieldName(String),
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
			items.extend(keyword_completions());
		}
		Context::FieldName(ref table) => {
			if let Some(sg) = schema {
				for field in sg.fields_of(table) {
					let detail = field.kind.clone().or_else(|| Some("any".into()));
					items.push(CompletionItem {
						label: field.name.clone(),
						kind: Some(CompletionItemKind::FIELD),
						detail,
						..Default::default()
					});
				}
				// Also suggest graph traversals
				items.push(CompletionItem {
					label: "->".into(),
					kind: Some(CompletionItemKind::OPERATOR),
					detail: Some("graph traversal (outgoing)".into()),
					..Default::default()
				});
				items.push(CompletionItem {
					label: "<-".into(),
					kind: Some(CompletionItemKind::OPERATOR),
					detail: Some("graph traversal (incoming)".into()),
					..Default::default()
				});
			}
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
					// Snippet: insert fn name + parentheses with tab stops
					let insert_text = func.map(|f| {
						if f.args.is_empty() {
							format!("{name}()")
						} else {
							let params: Vec<String> = f
								.args
								.iter()
								.enumerate()
								.map(|(i, (n, _))| format!("${{{}: {}}}", i + 1, n))
								.collect();
							format!("{name}({})", params.join(", "))
						}
					});
					items.push(CompletionItem {
						label: format!("fn::{name}"),
						kind: Some(CompletionItemKind::FUNCTION),
						detail,
						insert_text,
						insert_text_format: Some(InsertTextFormat::SNIPPET),
						..Default::default()
					});
				}
			}
			// Built-in function namespaces
			for ns in &[
				"array", "count", "crypto", "duration", "geo", "http", "math", "meta", "object",
				"parse", "rand", "search", "session", "sleep", "string", "time", "type",
			] {
				items.push(CompletionItem {
					label: format!("{ns}::"),
					kind: Some(CompletionItemKind::MODULE),
					detail: Some("built-in namespace".into()),
					..Default::default()
				});
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
			// Also suggest params from current document
			if let Ok(params) = surql_parser::extract_params(source) {
				for name in params {
					items.push(CompletionItem {
						label: format!("${name}"),
						kind: Some(CompletionItemKind::VARIABLE),
						detail: Some("query parameter".into()),
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

	// After `identifier.` — field completion
	if trimmed.ends_with('.') {
		let before_dot = &trimmed[..trimmed.len() - 1];
		let table_name = extract_last_identifier(before_dot);
		if !table_name.is_empty() {
			return Context::FieldName(table_name);
		}
	}

	// After FROM, INTO, ON — table completion
	let upper_raw = before_cursor.to_uppercase();
	let upper_trimmed = trimmed.to_uppercase();
	for keyword in &["FROM", "INTO", "ON", "TABLE"] {
		if upper_raw.ends_with(&format!("{keyword} "))
			|| upper_raw.ends_with(&format!("{keyword}\t"))
		{
			return Context::TableName;
		}
		if upper_trimmed.ends_with(keyword) {
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

/// Extract the last identifier before a position (scanning backwards).
fn extract_last_identifier(s: &str) -> String {
	let bytes = s.as_bytes();
	let end = bytes.len();
	let start = (0..end)
		.rev()
		.take_while(|&i| bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_')
		.last()
		.unwrap_or(end);
	if start < end {
		s[start..end].to_string()
	} else {
		String::new()
	}
}
