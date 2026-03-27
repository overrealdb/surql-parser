//! Signature help — show function parameter info while typing.

use surql_parser::SchemaGraph;
use tower_lsp::lsp_types::*;

/// Provide signature help at cursor position.
pub fn signature_help(
	source: &str,
	position: Position,
	schema: Option<&SchemaGraph>,
) -> Option<SignatureHelp> {
	let line = source.lines().nth(position.line as usize)?;
	let col = position.character as usize;
	let before = if col <= line.len() {
		&line[..col]
	} else {
		line
	};

	let (full_name, active_param) = find_function_call(before)?;

	// Try user-defined function (fn::name)
	if let Some(fn_name) = full_name.strip_prefix("fn::")
		&& let Some(sg) = schema
		&& let Some(func) = sg.function(fn_name)
	{
		return Some(build_user_fn_signature(func, active_param));
	}

	// Try built-in function (string::len, array::add, etc.)
	if let Some(builtin) = surql_parser::builtin_function(&full_name) {
		return Some(build_builtin_signature(builtin, active_param));
	}

	None
}

fn build_user_fn_signature(
	func: &surql_parser::schema_graph::FunctionDef,
	active_param: u32,
) -> SignatureHelp {
	let params: Vec<ParameterInformation> = func
		.args
		.iter()
		.map(|(name, kind)| ParameterInformation {
			label: ParameterLabel::Simple(format!("{name}: {kind}")),
			documentation: None,
		})
		.collect();

	let args_str = func
		.args
		.iter()
		.map(|(n, t)| format!("{n}: {t}"))
		.collect::<Vec<_>>()
		.join(", ");
	let ret = func
		.returns
		.as_ref()
		.map(|r| format!(" -> {r}"))
		.unwrap_or_default();
	let label = format!("fn::{}({args_str}){ret}", func.name);

	SignatureHelp {
		signatures: vec![SignatureInformation {
			label,
			documentation: None,
			parameters: Some(params),
			active_parameter: Some(active_param),
		}],
		active_signature: Some(0),
		active_parameter: Some(active_param),
	}
}

fn build_builtin_signature(
	builtin: &surql_parser::builtins_generated::BuiltinFn,
	active_param: u32,
) -> SignatureHelp {
	let signatures: Vec<SignatureInformation> = builtin
		.signatures
		.iter()
		.map(|sig| {
			let params = parse_signature_params(sig);
			SignatureInformation {
				label: sig.to_string(),
				documentation: Some(Documentation::String(builtin.description.to_string())),
				parameters: Some(params),
				active_parameter: Some(active_param),
			}
		})
		.collect();

	if signatures.is_empty() {
		return SignatureHelp {
			signatures: vec![SignatureInformation {
				label: builtin.name.to_string(),
				documentation: Some(Documentation::String(builtin.description.to_string())),
				parameters: Some(Vec::new()),
				active_parameter: Some(0),
			}],
			active_signature: Some(0),
			active_parameter: Some(active_param),
		};
	}

	SignatureHelp {
		signatures,
		active_signature: Some(0),
		active_parameter: Some(active_param),
	}
}

/// Parse parameter info from a signature string like `"string::len(string) -> number"`.
fn parse_signature_params(sig: &str) -> Vec<ParameterInformation> {
	let paren_start = match sig.find('(') {
		Some(i) => i,
		None => return Vec::new(),
	};
	let paren_end = match sig.rfind(')') {
		Some(i) => i,
		None => return Vec::new(),
	};
	let params_str = &sig[paren_start + 1..paren_end];
	if params_str.trim().is_empty() {
		return Vec::new();
	}

	split_params(params_str)
		.iter()
		.map(|p| ParameterInformation {
			label: ParameterLabel::Simple(p.trim().to_string()),
			documentation: None,
		})
		.collect()
}

/// Split parameter list respecting nested `<>` and `()` brackets.
pub(crate) fn split_params(s: &str) -> Vec<String> {
	let mut params = Vec::new();
	let mut depth = 0i32;
	let mut current = String::new();

	for ch in s.chars() {
		match ch {
			'<' | '(' => {
				depth += 1;
				current.push(ch);
			}
			'>' | ')' => {
				depth -= 1;
				current.push(ch);
			}
			',' if depth == 0 => {
				let trimmed = current.trim().to_string();
				if !trimmed.is_empty() {
					params.push(trimmed);
				}
				current.clear();
			}
			_ => current.push(ch),
		}
	}

	let trimmed = current.trim().to_string();
	if !trimmed.is_empty() {
		params.push(trimmed);
	}

	params
}

/// Find the function name and active parameter index at cursor.
fn find_function_call(before_cursor: &str) -> Option<(String, u32)> {
	let bytes = before_cursor.as_bytes();
	let mut depth = 0i32;
	let mut commas = 0u32;

	for i in (0..bytes.len()).rev() {
		match bytes[i] {
			b')' => depth += 1,
			b'(' => {
				if depth == 0 {
					let before_paren = &before_cursor[..i];
					let fn_name = extract_fn_name(before_paren)?;
					return Some((fn_name, commas));
				}
				depth -= 1;
			}
			b',' if depth == 0 => commas += 1,
			_ => {}
		}
	}
	None
}

/// Extract a function name (with namespace) from text ending just before `(`.
fn extract_fn_name(s: &str) -> Option<String> {
	let trimmed = s.trim_end();
	let bytes = trimmed.as_bytes();
	let end = bytes.len();
	let start = (0..end)
		.rev()
		.take_while(|&i| bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b':')
		.last()
		.unwrap_or(end);

	let candidate = &trimmed[start..end];
	if !candidate.contains("::") || candidate.is_empty() {
		return None;
	}
	Some(candidate.to_string())
}
