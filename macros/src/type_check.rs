use quote::quote;

/// Returns the list of Rust types that are compatible with a given SurrealQL type string.
pub(crate) fn expected_rust_types_for_surql(surql_type: &str) -> Option<&'static [&'static str]> {
	match surql_type.to_lowercase().as_str() {
		"string" => Some(&["&str", "String", "&String", "Cow<str>", "Cow<'_, str>"]),
		"int" => Some(&[
			"i64", "i32", "i16", "i8", "u64", "u32", "u16", "u8", "isize", "usize",
		]),
		"float" => Some(&["f64", "f32"]),
		"bool" => Some(&["bool"]),
		"datetime" => Some(&["DateTime", "Datetime", "NaiveDateTime", "chrono::DateTime"]),
		"duration" => Some(&["Duration", "std::time::Duration"]),
		"object" => Some(&[
			"Value",
			"Object",
			"Map",
			"BTreeMap",
			"HashMap",
			"serde_json::Value",
		]),
		"array" => Some(&["Vec", "Array", "&[", "Slice"]),
		"number" => Some(&[
			"f64", "f32", "i64", "i32", "u64", "u32", "Decimal", "Number",
		]),
		"decimal" => Some(&["Decimal", "rust_decimal::Decimal"]),
		"bytes" => Some(&["Vec<u8>", "Bytes", "&[u8]"]),
		"uuid" => Some(&["Uuid", "uuid::Uuid"]),
		"record" => Some(&["RecordId", "Thing", "&str", "String"]),
		"regex" => Some(&["Regex", "regex::Regex", "&str", "String"]),
		"none" => Some(&["()", "Option"]),
		"null" => Some(&["()", "Option"]),
		"any" => None,
		_ => None,
	}
}

/// Check whether a SurrealQL type is compatible with a Rust type string.
///
/// Returns `true` if the types are compatible, `false` if they definitely conflict.
/// Unknown SurrealQL types always return `true` (don't block compilation).
pub(crate) fn surql_type_matches_rust(surql_type: &str, rust_type: &str) -> bool {
	let normalized_owned = surql_type.to_lowercase();
	let normalized = normalized_owned.trim();

	// Handle union types: "none | string" means Option<String>
	if normalized.contains('|') {
		let parts: Vec<&str> = normalized.split('|').map(|s| s.trim()).collect();
		let has_none = parts.iter().any(|p| *p == "none" || *p == "null");
		let non_none: Vec<&str> = parts
			.iter()
			.filter(|p| **p != "none" && **p != "null")
			.copied()
			.collect();

		if has_none {
			// "none | T" should map to Option<RustT>
			if rust_type.contains("Option") {
				// Check inner type if there's exactly one non-none part
				if non_none.len() == 1 {
					let inner = extract_option_inner(rust_type);
					if let Some(inner) = inner {
						return surql_type_matches_rust(non_none[0], inner);
					}
				}
				return true;
			}
			// Also allow the non-none type directly (SurrealQL is lenient)
			if non_none.len() == 1 {
				return surql_type_matches_rust(non_none[0], rust_type);
			}
		}
		// Multiple union types: any single match is OK
		return parts.iter().any(|p| surql_type_matches_rust(p, rust_type));
	}

	// Handle option<T>
	if let Some(inner) = strip_wrapper(normalized, "option<", ">") {
		if rust_type.contains("Option") {
			let rust_inner = extract_option_inner(rust_type);
			if let Some(rust_inner) = rust_inner {
				return surql_type_matches_rust(inner, rust_inner);
			}
			return true;
		}
		// Also allow the inner type directly
		return surql_type_matches_rust(inner, rust_type);
	}

	// Handle record<table>
	if normalized.starts_with("record<") || normalized == "record" {
		return rust_type.contains("RecordId")
			|| rust_type.contains("Thing")
			|| rust_type.contains("&str")
			|| rust_type.contains("String")
			|| rust_type.contains("str");
	}

	// Handle array<T> and set<T>
	if let Some(inner) = strip_wrapper(normalized, "array<", ">") {
		if !rust_type.contains("Vec") && !rust_type.contains("Array") && !rust_type.contains("&[") {
			return false;
		}
		let rust_inner = extract_generic_inner(rust_type, "Vec");
		if let Some(rust_inner) = rust_inner {
			return surql_type_matches_rust(inner, rust_inner);
		}
		return true;
	}

	if let Some(inner) = strip_wrapper(normalized, "set<", ">") {
		if !rust_type.contains("Vec")
			&& !rust_type.contains("Set")
			&& !rust_type.contains("BTreeSet")
			&& !rust_type.contains("HashSet")
		{
			return false;
		}
		let rust_inner = extract_generic_inner(rust_type, "Vec")
			.or_else(|| extract_generic_inner(rust_type, "BTreeSet"))
			.or_else(|| extract_generic_inner(rust_type, "HashSet"));
		if let Some(rust_inner) = rust_inner {
			return surql_type_matches_rust(inner, rust_inner);
		}
		return true;
	}

	// Simple type matching
	match normalized {
		"string" => {
			rust_type.contains("str") || rust_type.contains("String") || rust_type.contains("Cow")
		}
		"int" => matches!(
			rust_type,
			"i64" | "i32" | "i16" | "i8" | "u64" | "u32" | "u16" | "u8" | "isize" | "usize"
		),
		"float" => matches!(rust_type, "f64" | "f32"),
		"bool" => rust_type == "bool",
		"datetime" => {
			rust_type.contains("DateTime")
				|| rust_type.contains("Datetime")
				|| rust_type.contains("NaiveDateTime")
		}
		"duration" => rust_type.contains("Duration"),
		"object" => {
			rust_type.contains("Value")
				|| rust_type.contains("Object")
				|| rust_type.contains("Map")
				|| rust_type.contains("BTreeMap")
				|| rust_type.contains("HashMap")
		}
		"array" => {
			rust_type.contains("Vec") || rust_type.contains("Array") || rust_type.contains("&[")
		}
		"number" => {
			matches!(rust_type, "f64" | "f32" | "i64" | "i32" | "u64" | "u32")
				|| rust_type.contains("Decimal")
				|| rust_type.contains("Number")
		}
		"decimal" => rust_type.contains("Decimal"),
		"bytes" => {
			rust_type.contains("Vec<u8>")
				|| rust_type.contains("Bytes")
				|| rust_type.contains("&[u8]")
		}
		"uuid" => rust_type.contains("Uuid"),
		"record" => {
			rust_type.contains("RecordId")
				|| rust_type.contains("Thing")
				|| rust_type.contains("&str")
				|| rust_type.contains("String")
				|| rust_type.contains("str")
		}
		"regex" => {
			rust_type.contains("Regex") || rust_type.contains("str") || rust_type.contains("String")
		}
		"any" => true,
		"none" | "null" => rust_type == "()" || rust_type.contains("Option"),
		_ => true, // unknown SurrealQL types pass (don't block compilation)
	}
}

/// Strip a wrapper like "array<inner>" and return "inner".
pub(crate) fn strip_wrapper<'a>(s: &'a str, prefix: &str, suffix: &str) -> Option<&'a str> {
	if s.starts_with(prefix) && s.ends_with(suffix) {
		Some(&s[prefix.len()..s.len() - suffix.len()])
	} else {
		None
	}
}

/// Extract the inner type from `Option<T>` → `T`.
pub(crate) fn extract_option_inner(rust_type: &str) -> Option<&str> {
	let trimmed = rust_type.trim();
	if let Some(rest) = trimmed.strip_prefix("Option<")
		&& let Some(inner) = rest.strip_suffix('>')
	{
		return Some(inner);
	}
	if let Some(rest) = trimmed.strip_prefix("Option <")
		&& let Some(inner) = rest.strip_suffix('>')
	{
		return Some(inner);
	}
	None
}

/// Extract the inner type from `Vec<T>`, `BTreeSet<T>`, etc.
pub(crate) fn extract_generic_inner<'a>(rust_type: &'a str, wrapper: &str) -> Option<&'a str> {
	let trimmed = rust_type.trim();
	let prefix = format!("{wrapper}<");
	if let Some(rest) = trimmed.strip_prefix(&prefix)
		&& let Some(inner) = rest.strip_suffix('>')
	{
		return Some(inner);
	}
	// Handle with space: "Vec <T>"
	let prefix_space = format!("{wrapper} <");
	if let Some(rest) = trimmed.strip_prefix(&prefix_space)
		&& let Some(inner) = rest.strip_suffix('>')
	{
		return Some(inner);
	}
	None
}

/// Format the Rust type string from a syn::Type, normalizing whitespace.
pub(crate) fn format_rust_type(ty: &syn::Type) -> String {
	let raw = quote!(#ty).to_string();
	// Normalize spaces around angle brackets and ampersands for reliable matching.
	// proc-macro2 inserts spaces: "& str" → "&str", "Vec < u8 >" → "Vec<u8>"
	normalize_type_string(&raw)
}

/// Normalize a type string by collapsing whitespace around punctuation.
pub(crate) fn normalize_type_string(s: &str) -> String {
	let mut result = String::with_capacity(s.len());
	let chars: Vec<char> = s.chars().collect();
	let len = chars.len();
	let mut i = 0;

	while i < len {
		let c = chars[i];
		match c {
			'<' => {
				// Remove trailing space before '<' in result
				while result.ends_with(' ') {
					result.pop();
				}
				result.push('<');
				// Skip leading spaces after '<'
				i += 1;
				while i < len && chars[i] == ' ' {
					i += 1;
				}
			}
			'>' => {
				// Remove trailing space before '>'
				while result.ends_with(' ') {
					result.pop();
				}
				result.push('>');
				i += 1;
			}
			'[' => {
				// Remove trailing space before '['
				while result.ends_with(' ') {
					result.pop();
				}
				result.push('[');
				// Skip leading spaces after '['
				i += 1;
				while i < len && chars[i] == ' ' {
					i += 1;
				}
			}
			']' => {
				// Remove trailing space before ']'
				while result.ends_with(' ') {
					result.pop();
				}
				result.push(']');
				i += 1;
			}
			'&' => {
				result.push('&');
				i += 1;
				// Skip space after '&' for references: "& str" → "&str"
				while i < len && chars[i] == ' ' {
					i += 1;
				}
			}
			',' => {
				// Remove trailing space before ','
				while result.ends_with(' ') {
					result.pop();
				}
				result.push(',');
				i += 1;
				// Skip spaces after ','
				while i < len && chars[i] == ' ' {
					i += 1;
				}
				// Add single space after comma for readability
				if i < len && chars[i] != ' ' {
					result.push(' ');
				}
			}
			' ' => {
				// Collapse multiple spaces into one, but skip spaces adjacent to punctuation
				if !result.is_empty()
					&& !result.ends_with(' ')
					&& !result.ends_with('<')
					&& !result.ends_with('&')
				{
					// Peek ahead: if next non-space is '>' or '<' or ',', skip the space
					let mut j = i + 1;
					while j < len && chars[j] == ' ' {
						j += 1;
					}
					if j < len && !matches!(chars[j], '>' | '<' | ',' | ']' | '[') {
						result.push(' ');
					}
				}
				i += 1;
			}
			_ => {
				result.push(c);
				i += 1;
			}
		}
	}

	result
}

/// Extract typed parameter names and their Rust type strings from a function signature.
pub(crate) fn extract_rust_param_types(func: &syn::ItemFn) -> Vec<(String, String)> {
	func.sig
		.inputs
		.iter()
		.filter_map(|arg| {
			if let syn::FnArg::Typed(pat_type) = arg {
				let name = quote!(#pat_type).to_string();
				// Extract just the parameter name (before the ':')
				let param_name = name.split(':').next().unwrap_or("").trim().to_string();
				let ty_str = format_rust_type(&pat_type.ty);
				Some((param_name, ty_str))
			} else {
				None
			}
		})
		.collect()
}

/// Build a human-readable hint for which Rust types match a SurrealQL type.
pub(crate) fn type_hint_for_surql(surql_type: &str) -> String {
	let normalized = surql_type.to_lowercase();
	let normalized = normalized.trim();

	// Handle union types
	if normalized.contains('|') {
		let parts: Vec<&str> = normalized.split('|').map(|s| s.trim()).collect();
		let has_none = parts.iter().any(|p| *p == "none" || *p == "null");
		let non_none: Vec<&str> = parts
			.iter()
			.filter(|p| **p != "none" && **p != "null")
			.copied()
			.collect();

		if has_none && non_none.len() == 1 {
			let inner_hint = type_hint_for_surql(non_none[0]);
			return format!("Option<{inner_hint}> (nullable field)");
		}
	}

	// Handle record<T>
	if normalized.starts_with("record<") {
		let table = &normalized[7..normalized.len().saturating_sub(1)];
		return format!("RecordId, Thing, String, &str (record reference to `{table}`)");
	}

	// Handle array<T>
	if normalized.starts_with("array<") {
		let inner = &normalized[6..normalized.len().saturating_sub(1)];
		let inner_hint = type_hint_for_surql(inner);
		return format!("Vec<{inner_hint}>");
	}

	// Handle set<T>
	if normalized.starts_with("set<") {
		let inner = &normalized[4..normalized.len().saturating_sub(1)];
		let inner_hint = type_hint_for_surql(inner);
		return format!("Vec<{inner_hint}>, BTreeSet, HashSet");
	}

	// Handle option<T>
	if normalized.starts_with("option<") {
		let inner = &normalized[7..normalized.len().saturating_sub(1)];
		let inner_hint = type_hint_for_surql(inner);
		return format!("Option<{inner_hint}>");
	}

	if let Some(expected) = expected_rust_types_for_surql(normalized) {
		expected.join(", ")
	} else {
		"any Rust type".to_string()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_match_string_types() {
		assert!(surql_type_matches_rust("string", "&str"));
		assert!(surql_type_matches_rust("string", "String"));
		assert!(surql_type_matches_rust("string", "&String"));
		assert!(!surql_type_matches_rust("string", "i64"));
		assert!(!surql_type_matches_rust("string", "bool"));
		assert!(!surql_type_matches_rust("string", "Vec<u8>"));
	}

	#[test]
	fn should_match_int_types() {
		assert!(surql_type_matches_rust("int", "i64"));
		assert!(surql_type_matches_rust("int", "i32"));
		assert!(surql_type_matches_rust("int", "u64"));
		assert!(surql_type_matches_rust("int", "u32"));
		assert!(surql_type_matches_rust("int", "usize"));
		assert!(surql_type_matches_rust("int", "isize"));
		assert!(!surql_type_matches_rust("int", "f64"));
		assert!(!surql_type_matches_rust("int", "&str"));
		assert!(!surql_type_matches_rust("int", "bool"));
	}

	#[test]
	fn should_match_float_types() {
		assert!(surql_type_matches_rust("float", "f64"));
		assert!(surql_type_matches_rust("float", "f32"));
		assert!(!surql_type_matches_rust("float", "i64"));
		assert!(!surql_type_matches_rust("float", "String"));
	}

	#[test]
	fn should_match_bool_type() {
		assert!(surql_type_matches_rust("bool", "bool"));
		assert!(!surql_type_matches_rust("bool", "i64"));
		assert!(!surql_type_matches_rust("bool", "&str"));
	}

	#[test]
	fn should_match_datetime_types() {
		assert!(surql_type_matches_rust("datetime", "DateTime<Utc>"));
		assert!(surql_type_matches_rust("datetime", "Datetime"));
		assert!(surql_type_matches_rust("datetime", "NaiveDateTime"));
		assert!(!surql_type_matches_rust("datetime", "String"));
		assert!(!surql_type_matches_rust("datetime", "i64"));
	}

	#[test]
	fn should_match_duration_types() {
		assert!(surql_type_matches_rust("duration", "Duration"));
		assert!(surql_type_matches_rust("duration", "std::time::Duration"));
		assert!(!surql_type_matches_rust("duration", "i64"));
	}

	#[test]
	fn should_match_object_types() {
		assert!(surql_type_matches_rust("object", "Value"));
		assert!(surql_type_matches_rust("object", "Object"));
		assert!(surql_type_matches_rust("object", "Map<String, Value>"));
		assert!(surql_type_matches_rust("object", "BTreeMap<String, Value>"));
		assert!(surql_type_matches_rust("object", "HashMap<String, Value>"));
		assert!(!surql_type_matches_rust("object", "Vec<u8>"));
		assert!(!surql_type_matches_rust("object", "i64"));
	}

	#[test]
	fn should_match_array_types() {
		assert!(surql_type_matches_rust("array", "Vec<String>"));
		assert!(surql_type_matches_rust("array", "Array"));
		assert!(!surql_type_matches_rust("array", "String"));
		assert!(!surql_type_matches_rust("array", "i64"));
	}

	#[test]
	fn should_match_number_types() {
		assert!(surql_type_matches_rust("number", "f64"));
		assert!(surql_type_matches_rust("number", "i64"));
		assert!(surql_type_matches_rust("number", "i32"));
		assert!(surql_type_matches_rust("number", "u64"));
		assert!(surql_type_matches_rust("number", "Decimal"));
		assert!(!surql_type_matches_rust("number", "String"));
		assert!(!surql_type_matches_rust("number", "bool"));
	}

	#[test]
	fn should_match_decimal_types() {
		assert!(surql_type_matches_rust("decimal", "Decimal"));
		assert!(surql_type_matches_rust("decimal", "rust_decimal::Decimal"));
		assert!(!surql_type_matches_rust("decimal", "f64"));
		assert!(!surql_type_matches_rust("decimal", "i64"));
	}

	#[test]
	fn should_match_bytes_types() {
		assert!(surql_type_matches_rust("bytes", "Vec<u8>"));
		assert!(surql_type_matches_rust("bytes", "Bytes"));
		assert!(surql_type_matches_rust("bytes", "&[u8]"));
		assert!(!surql_type_matches_rust("bytes", "String"));
	}

	#[test]
	fn should_match_uuid_types() {
		assert!(surql_type_matches_rust("uuid", "Uuid"));
		assert!(surql_type_matches_rust("uuid", "uuid::Uuid"));
		assert!(!surql_type_matches_rust("uuid", "String"));
		assert!(!surql_type_matches_rust("uuid", "i64"));
	}

	#[test]
	fn should_match_record_types() {
		assert!(surql_type_matches_rust("record", "RecordId"));
		assert!(surql_type_matches_rust("record", "Thing"));
		assert!(surql_type_matches_rust("record", "&str"));
		assert!(surql_type_matches_rust("record", "String"));
		assert!(!surql_type_matches_rust("record", "i64"));
		assert!(!surql_type_matches_rust("record", "bool"));
	}

	#[test]
	fn should_match_record_with_table() {
		assert!(surql_type_matches_rust("record<user>", "RecordId"));
		assert!(surql_type_matches_rust("record<user>", "String"));
		assert!(surql_type_matches_rust("record<user>", "&str"));
		assert!(!surql_type_matches_rust("record<user>", "i64"));
	}

	#[test]
	fn should_match_typed_array() {
		assert!(surql_type_matches_rust("array<string>", "Vec<String>"));
		assert!(surql_type_matches_rust("array<string>", "Vec<&str>"));
		assert!(surql_type_matches_rust("array<int>", "Vec<i64>"));
		assert!(!surql_type_matches_rust("array<string>", "Vec<i64>"));
		assert!(!surql_type_matches_rust("array<int>", "String"));
	}

	#[test]
	fn should_match_union_types() {
		// none | string → Option<String>
		assert!(surql_type_matches_rust("none | string", "Option<String>"));
		assert!(surql_type_matches_rust("none | string", "Option<&str>"));
		// Also allow bare String (SurrealQL is lenient)
		assert!(surql_type_matches_rust("none | string", "String"));
		// Wrong inner type
		assert!(!surql_type_matches_rust("none | string", "Option<i64>"));

		// none | datetime
		assert!(surql_type_matches_rust(
			"none | datetime",
			"Option<DateTime<Utc>>"
		));
	}

	#[test]
	fn should_match_option_types() {
		assert!(surql_type_matches_rust("option<string>", "Option<String>"));
		assert!(surql_type_matches_rust("option<int>", "Option<i64>"));
		assert!(!surql_type_matches_rust("option<string>", "Option<i64>"));
	}

	#[test]
	fn should_accept_any_type() {
		assert!(surql_type_matches_rust("any", "String"));
		assert!(surql_type_matches_rust("any", "i64"));
		assert!(surql_type_matches_rust("any", "bool"));
		assert!(surql_type_matches_rust("any", "Vec<u8>"));
	}

	#[test]
	fn should_accept_unknown_surql_types() {
		// Unknown types pass through (don't block compilation)
		assert!(surql_type_matches_rust("geometry", "String"));
		assert!(surql_type_matches_rust("custom_type", "i64"));
	}

	#[test]
	fn should_match_none_types() {
		assert!(surql_type_matches_rust("none", "()"));
		assert!(surql_type_matches_rust("none", "Option<String>"));
		assert!(!surql_type_matches_rust("none", "i64"));
		assert!(!surql_type_matches_rust("none", "String"));
	}

	#[test]
	fn should_normalize_type_strings() {
		assert_eq!(normalize_type_string("Vec < u8 >"), "Vec<u8>");
		assert_eq!(normalize_type_string("& str"), "&str");
		assert_eq!(normalize_type_string("Option < String >"), "Option<String>");
		assert_eq!(
			normalize_type_string("HashMap < String , Value >"),
			"HashMap<String, Value>"
		);
		assert_eq!(normalize_type_string("& [ u8 ]"), "&[u8]");
	}

	#[test]
	fn should_extract_option_inner() {
		assert_eq!(extract_option_inner("Option<String>"), Some("String"));
		assert_eq!(extract_option_inner("Option<i64>"), Some("i64"));
		assert_eq!(
			extract_option_inner("Option<DateTime<Utc>>"),
			Some("DateTime<Utc>")
		);
		assert_eq!(extract_option_inner("String"), None);
		assert_eq!(extract_option_inner("i64"), None);
	}

	#[test]
	fn should_extract_generic_inner() {
		assert_eq!(extract_generic_inner("Vec<String>", "Vec"), Some("String"));
		assert_eq!(extract_generic_inner("Vec<i64>", "Vec"), Some("i64"));
		assert_eq!(extract_generic_inner("Vec<u8>", "Vec"), Some("u8"));
		assert_eq!(
			extract_generic_inner("BTreeSet<String>", "BTreeSet"),
			Some("String")
		);
		assert_eq!(extract_generic_inner("String", "Vec"), None);
	}

	#[test]
	fn should_provide_type_hints() {
		let hint = type_hint_for_surql("string");
		assert!(hint.contains("&str"));
		assert!(hint.contains("String"));

		let hint = type_hint_for_surql("int");
		assert!(hint.contains("i64"));
		assert!(hint.contains("i32"));

		let hint = type_hint_for_surql("record<user>");
		assert!(hint.contains("RecordId"));
		assert!(hint.contains("user"));

		let hint = type_hint_for_surql("none | string");
		assert!(hint.contains("Option"));

		let hint = type_hint_for_surql("array<int>");
		assert!(hint.contains("Vec"));

		let hint = type_hint_for_surql("any");
		assert!(hint.contains("any Rust type"));
	}

	#[test]
	fn should_match_set_types() {
		assert!(surql_type_matches_rust("set<string>", "Vec<String>"));
		assert!(surql_type_matches_rust("set<string>", "BTreeSet<String>"));
		assert!(surql_type_matches_rust("set<string>", "HashSet<String>"));
		assert!(!surql_type_matches_rust("set<string>", "String"));
		assert!(!surql_type_matches_rust("set<int>", "Vec<String>"));
	}

	#[test]
	fn should_match_case_insensitive() {
		assert!(surql_type_matches_rust("String", "&str"));
		assert!(surql_type_matches_rust("STRING", "String"));
		assert!(surql_type_matches_rust("Int", "i64"));
		assert!(surql_type_matches_rust("FLOAT", "f64"));
		assert!(surql_type_matches_rust("Bool", "bool"));
	}
}
