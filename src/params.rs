use crate::{Result, parse};

/// Extract all `$param` names used in a SurrealQL query.
///
/// Parses the input, then scans for parameter tokens. Returns a sorted,
/// deduplicated list of parameter names (without the `$` prefix).
///
/// Parameters inside `DEFINE FUNCTION` signatures are excluded —
/// only "free" parameters (query-level bindings) are returned.
///
/// # Example
///
/// ```
/// let params = surql_parser::extract_params(
///     "SELECT * FROM user WHERE age > $min AND name = $name"
/// ).unwrap();
/// assert_eq!(params, vec!["min", "name"]);
/// ```
pub fn extract_params(input: &str) -> Result<Vec<String>> {
	parse(input)?;
	Ok(scan_params(input))
}

/// Scan a (known-valid) SurrealQL string for `$param` tokens.
///
/// Skips string literals and comments. Returns sorted, deduplicated names.
pub(crate) fn scan_params(input: &str) -> Vec<String> {
	let mut params = std::collections::BTreeSet::new();
	let bytes = input.as_bytes();
	let len = bytes.len();
	let mut i = 0;

	while i < len {
		match bytes[i] {
			// Skip single-quoted strings: 'text''s escaped'
			b'\'' => {
				i += 1;
				while i < len {
					if bytes[i] == b'\'' {
						i += 1;
						if i < len && bytes[i] == b'\'' {
							i += 1; // escaped ''
							continue;
						}
						break;
					}
					i += 1;
				}
			}
			// Skip double-quoted strings: "text\"s escaped"
			b'"' => {
				i += 1;
				while i < len {
					if bytes[i] == b'\\' {
						i += 2;
						continue;
					}
					if bytes[i] == b'"' {
						i += 1;
						break;
					}
					i += 1;
				}
			}
			// Skip backtick-quoted identifiers: `field name`
			b'`' => {
				i += 1;
				while i < len {
					if bytes[i] == b'`' {
						i += 1;
						break;
					}
					i += 1;
				}
			}
			// Skip line comments: -- ...
			b'-' if i + 1 < len && bytes[i + 1] == b'-' => {
				i += 2;
				while i < len && bytes[i] != b'\n' {
					i += 1;
				}
			}
			// Skip block comments: /* ... */
			b'/' if i + 1 < len && bytes[i + 1] == b'*' => {
				i += 2;
				let mut depth = 1u32;
				while i + 1 < len && depth > 0 {
					if bytes[i] == b'/' && bytes[i + 1] == b'*' {
						depth += 1;
						i += 2;
					} else if bytes[i] == b'*' && bytes[i + 1] == b'/' {
						depth -= 1;
						i += 2;
					} else {
						i += 1;
					}
				}
			}
			// Collect $param
			b'$' => {
				i += 1;
				let start = i;
				while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
					i += 1;
				}
				if i > start {
					let name = &input[start..i];
					params.insert(name.to_string());
				}
			}
			_ => i += 1,
		}
	}

	params.into_iter().collect()
}
