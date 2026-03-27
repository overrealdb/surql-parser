//! Extract embedded SurrealQL from Rust macro invocations.
//!
//! Finds surql_query!("..."), surql_check!("..."), and #[surql_function("fn::...")] in Rust source
//! and provides the SurrealQL content with its position in the host file.

/// A region of embedded SurrealQL within a Rust file.
#[derive(Debug, Clone, PartialEq)]
pub enum RegionKind {
	/// Full SurrealQL statement(s) — validate as SQL.
	Statement,
	/// Function name only (from #[surql_function]) — don't parse as SQL.
	FunctionName,
}

#[derive(Debug, Clone)]
pub struct EmbeddedRegion {
	/// The SurrealQL content (without quotes).
	pub content: String,
	/// Byte offset of the opening quote in the host file.
	pub offset: usize,
	/// Line number (0-indexed) of the opening quote.
	pub line: u32,
	/// Column (0-indexed) of the first char of content (after the quote).
	pub col: u32,
	/// What kind of embedded content this is.
	pub kind: RegionKind,
}

/// Extract all embedded SurrealQL regions from Rust source.
pub fn extract_surql_from_rust(source: &str) -> Vec<EmbeddedRegion> {
	let mut regions = Vec::new();
	let bytes = source.as_bytes();
	let len = bytes.len();
	let mut i = 0;

	while i < len {
		// Look for surql_query!( or surql_check!(
		if let Some(macro_end) = find_surql_macro(bytes, i)
			&& let Some(region) = extract_string_after(source, macro_end, RegionKind::Statement)
		{
			let skip = macro_end + region.content.len() + 2;
			regions.push(region);
			i = skip;
			continue;
		}

		// Look for #[surql_function("fn::...")]
		if bytes[i] == b'#'
			&& i + 1 < len
			&& bytes[i + 1] == b'['
			&& let Some(region) = find_surql_function_attr(source, i)
		{
			regions.push(region);
		}

		i += 1;
	}

	regions
}

fn find_surql_macro(bytes: &[u8], pos: usize) -> Option<usize> {
	for prefix in &[b"surql_query!(" as &[u8], b"surql_check!("] {
		if pos + prefix.len() <= bytes.len() && &bytes[pos..pos + prefix.len()] == *prefix {
			return Some(pos + prefix.len());
		}
	}
	None
}

fn extract_string_after(source: &str, pos: usize, kind: RegionKind) -> Option<EmbeddedRegion> {
	let bytes = source.as_bytes();
	let mut i = pos;

	while i < bytes.len() && bytes[i].is_ascii_whitespace() {
		i += 1;
	}

	if i >= bytes.len() || bytes[i] != b'"' {
		return None;
	}

	let content_start = i + 1;
	let mut j = content_start;
	while j < bytes.len() {
		if bytes[j] == b'\\' {
			j += 2;
			continue;
		}
		if bytes[j] == b'"' {
			let content = &source[content_start..j];
			let (line, col) = offset_to_line_col(source, content_start);
			return Some(EmbeddedRegion {
				content: content.to_string(),
				offset: content_start,
				line,
				col,
				kind,
			});
		}
		j += 1;
	}
	None
}

fn find_surql_function_attr(source: &str, hash_pos: usize) -> Option<EmbeddedRegion> {
	let rest = &source[hash_pos..];
	if !rest.starts_with("#[surql_function(") {
		return None;
	}
	let paren_start = hash_pos + "#[surql_function(".len();
	extract_string_after(source, paren_start, RegionKind::FunctionName)
}

fn offset_to_line_col(source: &str, offset: usize) -> (u32, u32) {
	let before = &source[..offset];
	let line = before.matches('\n').count() as u32;
	let col = before.rfind('\n').map(|i| offset - i - 1).unwrap_or(offset) as u32;
	(line, col)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_extract_surql_query_macro() {
		let source = r#"let q = surql_query!("SELECT * FROM user", name);"#;
		let regions = extract_surql_from_rust(source);
		assert_eq!(regions.len(), 1);
		assert_eq!(regions[0].content, "SELECT * FROM user");
		assert_eq!(regions[0].kind, RegionKind::Statement);
	}

	#[test]
	fn should_extract_surql_check_macro() {
		let source = r#"let q = surql_check!("DEFINE TABLE user SCHEMAFULL");"#;
		let regions = extract_surql_from_rust(source);
		assert_eq!(regions.len(), 1);
		assert_eq!(regions[0].content, "DEFINE TABLE user SCHEMAFULL");
		assert_eq!(regions[0].kind, RegionKind::Statement);
	}

	#[test]
	fn should_extract_surql_function_attr() {
		let source = r#"#[surql_function("fn::get_user")]
fn get_user() {}"#;
		let regions = extract_surql_from_rust(source);
		assert_eq!(regions.len(), 1);
		assert_eq!(regions[0].content, "fn::get_user");
		assert_eq!(regions[0].kind, RegionKind::FunctionName);
	}

	#[test]
	fn should_extract_multiple_regions() {
		let source = r#"
let a = surql_query!("SELECT * FROM user", name);
let b = surql_check!("CREATE post SET title = 'hi'");
"#;
		let regions = extract_surql_from_rust(source);
		assert_eq!(regions.len(), 2);
	}

	#[test]
	fn should_return_empty_for_no_macros() {
		let source = r#"fn main() { println!("hello"); }"#;
		let regions = extract_surql_from_rust(source);
		assert!(regions.is_empty());
	}

	#[test]
	fn should_track_correct_line_col() {
		let source = "line1\nlet q = surql_query!(\"SELECT 1\");";
		let regions = extract_surql_from_rust(source);
		assert_eq!(regions.len(), 1);
		assert_eq!(regions[0].line, 1);
		assert_eq!(regions[0].content, "SELECT 1");
	}
}
