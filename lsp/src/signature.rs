//! Signature help — show function parameter info while typing.

use surql_parser::SchemaGraph;
use tower_lsp::lsp_types::*;

/// Provide signature help at cursor position.
pub fn signature_help(
	source: &str,
	position: Position,
	schema: Option<&SchemaGraph>,
) -> Option<SignatureHelp> {
	let sg = schema?;
	let line = source.lines().nth(position.line as usize)?;
	let col = position.character as usize;
	let before = if col <= line.len() {
		&line[..col]
	} else {
		line
	};

	// Find the function call: scan backwards for `fn::name(`
	let (fn_name, active_param) = find_function_call(before)?;

	let func = sg.function(&fn_name)?;

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

	Some(SignatureHelp {
		signatures: vec![SignatureInformation {
			label,
			documentation: None,
			parameters: Some(params),
			active_parameter: Some(active_param),
		}],
		active_signature: Some(0),
		active_parameter: Some(active_param),
	})
}

/// Find the function name and active parameter index at cursor.
/// Returns `(function_name_without_fn_prefix, active_param_index)`.
fn find_function_call(before_cursor: &str) -> Option<(String, u32)> {
	// Look for pattern: fn::name(arg1, arg2, |cursor
	let bytes = before_cursor.as_bytes();
	let mut depth = 0i32;
	let mut commas = 0u32;

	// Scan backwards to find the opening `(`
	for i in (0..bytes.len()).rev() {
		match bytes[i] {
			b')' => depth += 1,
			b'(' => {
				if depth == 0 {
					// Found the opening paren
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

/// Extract `fn::name` from text ending just before `(`.
fn extract_fn_name(s: &str) -> Option<String> {
	let trimmed = s.trim_end();
	// Walk backwards to collect the function path
	let bytes = trimmed.as_bytes();
	let end = bytes.len();
	let start = (0..end)
		.rev()
		.take_while(|&i| bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b':')
		.last()
		.unwrap_or(end);

	let candidate = &trimmed[start..end];
	// Must start with fn::
	let name = candidate.strip_prefix("fn::")?;
	if name.is_empty() {
		return None;
	}
	Some(name.to_string())
}
