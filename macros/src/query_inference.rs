use crate::type_check::strip_wrapper;

/// Inferred type constraint for a query parameter.
pub(crate) struct ParamTypeConstraint {
	pub param_name: String,
	pub surql_type: String,
	pub source: String,
}

/// Extract simple WHERE clause type constraints from a SurrealQL query.
///
/// Looks for patterns like:
///   - `WHERE field = $param` -> param type = field type
///   - `WHERE field > $param` -> param is numeric
///   - `WHERE field IN $param` -> param type = array of field type
///   - `WHERE field CONTAINS $param` -> param type = element type of field
///
/// Returns a list of (param_name, inferred_surql_type, source_description) tuples.
pub(crate) fn infer_param_types_from_query(
	sql: &str,
	field_types: &[(String, String, String)],
) -> Vec<ParamTypeConstraint> {
	let mut constraints = Vec::new();

	// Tokenize the query into a flat list for pattern matching
	let tokens = tokenize_for_inference(sql);

	let len = tokens.len();
	let mut i = 0;

	while i + 2 < len {
		let field = &tokens[i];
		let op = &tokens[i + 1];
		let value = &tokens[i + 2];

		// Pattern: field OP $param
		if !field.starts_with('$') && value.starts_with('$') {
			let param_name = &value[1..];

			// Find the table context (scan backwards for FROM <table>)
			let table = find_table_context(&tokens, i);

			// Look up the field type in the schema
			let field_type = table.as_ref().and_then(|tbl| {
				field_types
					.iter()
					.find(|(t, f, _)| t == tbl && f == field)
					.map(|(_, _, k)| k.as_str())
			});

			let op_lower = op.to_uppercase();
			match op_lower.as_str() {
				"=" | "==" | "!=" | "IS" => {
					if let Some(ft) = field_type {
						constraints.push(ParamTypeConstraint {
							param_name: param_name.to_string(),
							surql_type: ft.to_string(),
							source: format!(
								"`{field} {op} ${param_name}` (field `{field}` is `{ft}`)"
							),
						});
					}
				}
				">" | ">=" | "<" | "<=" => {
					if let Some(ft) = field_type {
						// For comparison ops, the field type must be numeric-compatible
						constraints.push(ParamTypeConstraint {
							param_name: param_name.to_string(),
							surql_type: ft.to_string(),
							source: format!(
								"`{field} {op} ${param_name}` (field `{field}` is `{ft}`)"
							),
						});
					} else {
						// No schema info, but comparison implies numeric
						constraints.push(ParamTypeConstraint {
							param_name: param_name.to_string(),
							surql_type: "number".to_string(),
							source: format!(
								"`{field} {op} ${param_name}` (comparison implies numeric)"
							),
						});
					}
				}
				"IN" | "INSIDE" => {
					// $param IN field -> param is element type
					// field IN $param -> $param is array<field_type>
					// Here: field IN $param -> $param should be array<field_type>
					if let Some(ft) = field_type {
						constraints.push(ParamTypeConstraint {
							param_name: param_name.to_string(),
							surql_type: format!("array<{ft}>"),
							source: format!(
								"`{field} IN ${param_name}` (field `{field}` is `{ft}`, so param should be array)"
							),
						});
					} else {
						constraints.push(ParamTypeConstraint {
							param_name: param_name.to_string(),
							surql_type: "array".to_string(),
							source: format!("`{field} IN ${param_name}` (IN requires array)"),
						});
					}
				}
				"CONTAINS" | "CONTAINSALL" | "CONTAINSANY" | "CONTAINSNONE" => {
					if let Some(ft) = field_type {
						// field CONTAINS $param -> $param is the element type
						// If field is array<T>, param should be T
						let ft_lower = ft.to_lowercase();
						let element_type = strip_wrapper(&ft_lower, "array<", ">").unwrap_or(ft);
						constraints.push(ParamTypeConstraint {
							param_name: param_name.to_string(),
							surql_type: element_type.to_string(),
							source: format!("`{field} CONTAINS ${param_name}` (element of `{ft}`)"),
						});
					}
				}
				_ => {}
			}
		}

		// Pattern: $param OP field (reversed)
		if field.starts_with('$') && !value.starts_with('$') {
			let param_name = &field[1..];
			let actual_field = value;
			let op_lower = op.to_uppercase();

			let table = find_table_context(&tokens, i);
			let field_type = table.as_ref().and_then(|tbl| {
				field_types
					.iter()
					.find(|(t, f, _)| t == tbl && f == actual_field)
					.map(|(_, _, k)| k.as_str())
			});

			match op_lower.as_str() {
				"=" | "==" | "!=" | "IS" | ">" | ">=" | "<" | "<=" => {
					if let Some(ft) = field_type {
						constraints.push(ParamTypeConstraint {
							param_name: param_name.to_string(),
							surql_type: ft.to_string(),
							source: format!(
								"`${param_name} {op} {actual_field}` (field `{actual_field}` is `{ft}`)"
							),
						});
					}
				}
				"IN" | "INSIDE" => {
					// $param IN field -> $param is element type of field
					if let Some(ft) = field_type {
						let ft_lower = ft.to_lowercase();
						let element_type = strip_wrapper(&ft_lower, "array<", ">").unwrap_or(ft);
						constraints.push(ParamTypeConstraint {
							param_name: param_name.to_string(),
							surql_type: element_type.to_string(),
							source: format!(
								"`${param_name} IN {actual_field}` (element of `{ft}`)"
							),
						});
					}
				}
				_ => {}
			}
		}

		i += 1;
	}

	constraints
}

/// Simple tokenizer for type inference. Splits on whitespace and operators,
/// preserving $params, field names, and operators as distinct tokens.
pub(crate) fn tokenize_for_inference(sql: &str) -> Vec<String> {
	let mut tokens = Vec::new();
	let mut current = String::new();
	let bytes = sql.as_bytes();
	let len = bytes.len();
	let mut i = 0;

	while i < len {
		match bytes[i] {
			// Skip string literals
			b'\'' => {
				if !current.is_empty() {
					tokens.push(std::mem::take(&mut current));
				}
				i += 1;
				while i < len {
					if bytes[i] == b'\'' {
						i += 1;
						if i < len && bytes[i] == b'\'' {
							i += 1;
							continue;
						}
						break;
					}
					i += 1;
				}
			}
			b'"' => {
				if !current.is_empty() {
					tokens.push(std::mem::take(&mut current));
				}
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
			// Whitespace separates tokens
			b' ' | b'\t' | b'\n' | b'\r' => {
				if !current.is_empty() {
					tokens.push(std::mem::take(&mut current));
				}
				i += 1;
			}
			// Multi-char operators
			b'!' if i + 1 < len && bytes[i + 1] == b'=' => {
				if !current.is_empty() {
					tokens.push(std::mem::take(&mut current));
				}
				tokens.push("!=".to_string());
				i += 2;
			}
			b'>' if i + 1 < len && bytes[i + 1] == b'=' => {
				if !current.is_empty() {
					tokens.push(std::mem::take(&mut current));
				}
				tokens.push(">=".to_string());
				i += 2;
			}
			b'<' if i + 1 < len && bytes[i + 1] == b'=' => {
				if !current.is_empty() {
					tokens.push(std::mem::take(&mut current));
				}
				tokens.push("<=".to_string());
				i += 2;
			}
			b'=' if i + 1 < len && bytes[i + 1] == b'=' => {
				if !current.is_empty() {
					tokens.push(std::mem::take(&mut current));
				}
				tokens.push("==".to_string());
				i += 2;
			}
			// Single-char operators
			b'=' | b'>' | b'<' => {
				if !current.is_empty() {
					tokens.push(std::mem::take(&mut current));
				}
				tokens.push((bytes[i] as char).to_string());
				i += 1;
			}
			// Skip commas and parens as separators
			b',' | b'(' | b')' | b';' => {
				if !current.is_empty() {
					tokens.push(std::mem::take(&mut current));
				}
				i += 1;
			}
			// Comment: --
			b'-' if i + 1 < len && bytes[i + 1] == b'-' => {
				if !current.is_empty() {
					tokens.push(std::mem::take(&mut current));
				}
				while i < len && bytes[i] != b'\n' {
					i += 1;
				}
			}
			// Block comment: /* ... */
			b'/' if i + 1 < len && bytes[i + 1] == b'*' => {
				if !current.is_empty() {
					tokens.push(std::mem::take(&mut current));
				}
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
			// Everything else builds the current token
			_ => {
				current.push(bytes[i] as char);
				i += 1;
			}
		}
	}

	if !current.is_empty() {
		tokens.push(current);
	}

	tokens
}

/// Scan backwards through tokens to find the table name in `FROM <table>`.
pub(crate) fn find_table_context(tokens: &[String], pos: usize) -> Option<String> {
	let mut i = pos;
	while i > 0 {
		i -= 1;
		if tokens[i].to_uppercase() == "FROM" && i + 1 < tokens.len() {
			let table = &tokens[i + 1];
			// Skip the ONLY keyword
			if table.to_uppercase() == "ONLY" && i + 2 < tokens.len() {
				return Some(tokens[i + 2].clone());
			}
			return Some(table.clone());
		}
	}
	None
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_tokenize_simple_query() {
		let tokens = tokenize_for_inference("SELECT * FROM user WHERE age > $min");
		assert!(tokens.contains(&"SELECT".to_string()));
		assert!(tokens.contains(&"*".to_string()));
		assert!(tokens.contains(&"FROM".to_string()));
		assert!(tokens.contains(&"user".to_string()));
		assert!(tokens.contains(&"WHERE".to_string()));
		assert!(tokens.contains(&"age".to_string()));
		assert!(tokens.contains(&">".to_string()));
		assert!(tokens.contains(&"$min".to_string()));
	}

	#[test]
	fn should_find_table_context() {
		let tokens = tokenize_for_inference("SELECT * FROM user WHERE age > $min");
		// Find the position of "age" token
		let age_pos = tokens.iter().position(|t| t == "age").unwrap();
		let table = find_table_context(&tokens, age_pos);
		assert_eq!(table, Some("user".to_string()));
	}

	#[test]
	fn should_infer_param_types_equality() {
		let field_types = vec![
			("user".to_string(), "name".to_string(), "string".to_string()),
			("user".to_string(), "age".to_string(), "int".to_string()),
		];
		let constraints =
			infer_param_types_from_query("SELECT * FROM user WHERE name = $name", &field_types);
		assert_eq!(constraints.len(), 1);
		assert_eq!(constraints[0].param_name, "name");
		assert_eq!(constraints[0].surql_type, "string");
	}

	#[test]
	fn should_infer_param_types_comparison() {
		let field_types = vec![("user".to_string(), "age".to_string(), "int".to_string())];
		let constraints =
			infer_param_types_from_query("SELECT * FROM user WHERE age > $min", &field_types);
		assert_eq!(constraints.len(), 1);
		assert_eq!(constraints[0].param_name, "min");
		assert_eq!(constraints[0].surql_type, "int");
	}

	#[test]
	fn should_infer_param_types_in_operator() {
		let field_types = vec![(
			"user".to_string(),
			"status".to_string(),
			"string".to_string(),
		)];
		let constraints = infer_param_types_from_query(
			"SELECT * FROM user WHERE status IN $statuses",
			&field_types,
		);
		assert_eq!(constraints.len(), 1);
		assert_eq!(constraints[0].param_name, "statuses");
		assert_eq!(constraints[0].surql_type, "array<string>");
	}

	#[test]
	fn should_infer_param_types_multiple() {
		let field_types = vec![
			("user".to_string(), "name".to_string(), "string".to_string()),
			("user".to_string(), "age".to_string(), "int".to_string()),
		];
		let constraints = infer_param_types_from_query(
			"SELECT * FROM user WHERE age > $min AND name = $name",
			&field_types,
		);
		assert!(constraints.len() >= 2);
		let min_constraint = constraints.iter().find(|c| c.param_name == "min").unwrap();
		assert_eq!(min_constraint.surql_type, "int");
		let name_constraint = constraints.iter().find(|c| c.param_name == "name").unwrap();
		assert_eq!(name_constraint.surql_type, "string");
	}

	#[test]
	fn should_handle_no_schema_comparison() {
		let field_types: Vec<(String, String, String)> = vec![];
		let constraints =
			infer_param_types_from_query("SELECT * FROM user WHERE age > $min", &field_types);
		// Without schema info, comparison still implies numeric
		assert_eq!(constraints.len(), 1);
		assert_eq!(constraints[0].param_name, "min");
		assert_eq!(constraints[0].surql_type, "number");
	}
}
