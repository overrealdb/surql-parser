//! Completion provider — keywords + schema-aware suggestions.

use surql_parser::SchemaGraph;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, InsertTextFormat, Position};

use crate::keywords;

/// Context detected at cursor position.
#[derive(Debug, PartialEq)]
pub(crate) enum Context {
	/// After FROM, INTO, ON — suggest table names.
	TableName,
	/// After `table.` — suggest fields of that table.
	FieldName(String),
	/// After `fn::` — suggest function names.
	FunctionName,
	/// After a built-in namespace like `string::`, `array::`, etc.
	BuiltinNamespace(String),
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
			// Built-in function namespaces (from generated data)
			for ns in surql_parser::builtins_generated::BUILTIN_NAMESPACES {
				// Only show top-level namespaces (no :: in the name)
				if !ns.contains("::") {
					items.push(CompletionItem {
						label: format!("{ns}::"),
						kind: Some(CompletionItemKind::MODULE),
						detail: Some("built-in namespace".into()),
						..Default::default()
					});
				}
			}
		}
		Context::BuiltinNamespace(ref ns) => {
			for builtin in surql_parser::builtins_in_namespace(ns) {
				let short_name = builtin
					.name
					.strip_prefix(&format!("{ns}::"))
					.unwrap_or(builtin.name);
				let detail = if builtin.signatures.is_empty() {
					Some(builtin.description.to_string())
				} else {
					Some(builtin.signatures[0].to_string())
				};
				let insert_text = Some(builtin_snippet(short_name, builtin));
				items.push(CompletionItem {
					label: builtin.name.to_string(),
					kind: Some(CompletionItemKind::FUNCTION),
					detail,
					insert_text,
					insert_text_format: Some(InsertTextFormat::SNIPPET),
					..Default::default()
				});
			}
			// Sub-namespaces (e.g., after `string::` suggest `string::semver::`)
			let prefix = format!("{ns}::");
			for sub_ns in surql_parser::builtins_generated::BUILTIN_NAMESPACES {
				if let Some(rest) = sub_ns.strip_prefix(&prefix)
					&& !rest.contains("::")
				{
					items.push(CompletionItem {
						label: format!("{sub_ns}::"),
						kind: Some(CompletionItemKind::MODULE),
						detail: Some("built-in namespace".into()),
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
			// Top-level built-in namespaces in general context
			for ns in surql_parser::builtins_generated::BUILTIN_NAMESPACES {
				if !ns.contains("::") {
					items.push(CompletionItem {
						label: format!("{ns}::"),
						kind: Some(CompletionItemKind::MODULE),
						detail: Some("built-in namespace".into()),
						..Default::default()
					});
				}
			}
		}
	}

	items
}

/// Build a snippet with numbered tab stops from a builtin's signature.
/// e.g., `"len"` + sig `"string::len(string) -> number"` → `"len(${1:string})"`
fn builtin_snippet(
	short_name: &str,
	builtin: &surql_parser::builtins_generated::BuiltinFn,
) -> String {
	if let Some(sig) = builtin.signatures.first()
		&& let Some(paren_start) = sig.find('(')
		&& let Some(paren_end) = sig.rfind(')')
	{
		let params_str = &sig[paren_start + 1..paren_end];
		if params_str.trim().is_empty() {
			return format!("{short_name}()");
		}
		let params = crate::signature::split_params(params_str);
		let snippets: Vec<String> = params
			.iter()
			.enumerate()
			.map(|(i, p)| format!("${{{}: {}}}", i + 1, p.trim()))
			.collect();
		return format!("{short_name}({})", snippets.join(", "));
	}
	format!("{short_name}($0)")
}

fn keyword_completions() -> Vec<CompletionItem> {
	keywords::all_keywords()
		.iter()
		.map(|kw| CompletionItem {
			label: kw.to_string(),
			kind: Some(CompletionItemKind::KEYWORD),
			..Default::default()
		})
		.collect()
}

/// Detect the completion context using the lexer's token stream.
///
/// This is correct even for tokens inside strings/comments (the lexer handles
/// those properly), unlike text-heuristic approaches.
pub(crate) fn detect_context(source: &str, position: Position) -> Context {
	use surql_parser::upstream::syn::lexer::Lexer;
	use surql_parser::upstream::syn::token::TokenKind;

	let byte_offset = position_to_byte_offset(source, position);
	let bytes = source.as_bytes();
	if bytes.is_empty() || bytes.len() > u32::MAX as usize {
		return Context::General;
	}

	// Tokenize everything up to cursor
	let lexer = Lexer::new(bytes);
	let tokens: Vec<_> = lexer
		.take_while(|t| (t.span.offset as usize) < byte_offset)
		.collect();

	if tokens.is_empty() {
		return Context::General;
	}

	let last = tokens.last().unwrap();
	let last_end = last.span.offset as usize + last.span.len as usize;

	/// Get the source text of a token.
	fn token_text<'a>(
		source: &'a str,
		token: &surql_parser::upstream::syn::token::Token,
	) -> &'a str {
		let start = token.span.offset as usize;
		let end = (token.span.offset + token.span.len) as usize;
		if end <= source.len() {
			&source[start..end]
		} else {
			""
		}
	}

	// Cursor is right after a `$param` → param context
	if last.kind == TokenKind::Parameter {
		return Context::ParamName;
	}

	// Cursor is right after `.` → field context
	if last.kind == TokenKind::Dot && tokens.len() >= 2 {
		let prev = &tokens[tokens.len() - 2];
		if prev.kind == TokenKind::Identifier || matches!(prev.kind, TokenKind::Keyword(_)) {
			return Context::FieldName(token_text(source, prev).to_string());
		}
	}

	// Cursor is after last token (in whitespace/gap)
	if last_end <= byte_offset {
		let last_text = token_text(source, last).to_uppercase();
		match last_text.as_str() {
			"FROM" | "INTO" | "ON" | "TABLE" => return Context::TableName,
			_ => {}
		}

		// After `namespace::` — lexer emits Keyword/Identifier + PathSeperator
		if last.kind == TokenKind::PathSeperator && tokens.len() >= 2 {
			let prev_text = token_text(source, &tokens[tokens.len() - 2]);
			let prev_upper = prev_text.to_uppercase();
			if prev_upper == "FN" {
				return Context::FunctionName;
			}
			// Check for built-in namespace (string::, array::, etc.)
			let ns = prev_text.to_lowercase();
			if surql_parser::builtins_generated::BUILTIN_NAMESPACES.contains(&ns.as_str()) {
				return Context::BuiltinNamespace(ns);
			}
		}

		// Multi-level namespace: e.g., `string::semver::` →
		// tokens: [Keyword("string"), PathSep, Ident("semver"), PathSep]
		if last.kind == TokenKind::PathSeperator && tokens.len() >= 4 {
			let t3 = token_text(source, &tokens[tokens.len() - 2]).to_lowercase();
			if tokens[tokens.len() - 3].kind == TokenKind::PathSeperator {
				let t1 = token_text(source, &tokens[tokens.len() - 4]).to_lowercase();
				let combined = format!("{t1}::{t3}");
				if surql_parser::builtins_generated::BUILTIN_NAMESPACES.contains(&combined.as_str())
				{
					return Context::BuiltinNamespace(combined);
				}
			}
		}

		if last_text == "FN" {
			return Context::FunctionName;
		}
	}

	// Fallback: cursor right after `$` in source
	if byte_offset > 0 && byte_offset <= source.len() && bytes[byte_offset - 1] == b'$' {
		return Context::ParamName;
	}

	Context::General
}

/// Convert an LSP Position (0-indexed line/col) to a byte offset.
pub(crate) fn position_to_byte_offset(source: &str, position: Position) -> usize {
	let mut offset = 0;
	for (i, line) in source.lines().enumerate() {
		if i == position.line as usize {
			return offset + (position.character as usize).min(line.len());
		}
		offset += line.len() + 1; // +1 for \n
	}
	source.len()
}
