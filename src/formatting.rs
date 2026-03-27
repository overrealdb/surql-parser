//! Configurable SurrealQL formatter.
//!
//! Preserves comments, strings, and document structure while applying
//! configurable formatting rules via `FormatConfig`.
//!
//! Config is loaded from `.surqlformat.toml` in the workspace root,
//! or falls back to sensible defaults.

use serde::Deserialize;

/// Formatting configuration loaded from `.surqlformat.toml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FormatConfig {
	pub uppercase_keywords: bool,
	pub indent_style: IndentStyle,
	pub indent_width: u32,
	pub newline_after_semicolon: bool,
	pub newline_before_where: bool,
	pub newline_before_set: bool,
	pub newline_before_from: bool,
	pub trailing_semicolon: bool,
	pub collapse_blank_lines: bool,
	pub max_blank_lines: u32,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IndentStyle {
	Tab,
	Space,
}

impl Default for FormatConfig {
	fn default() -> Self {
		Self {
			uppercase_keywords: true,
			indent_style: IndentStyle::Tab,
			indent_width: 4,
			newline_after_semicolon: false,
			newline_before_where: false,
			newline_before_set: false,
			newline_before_from: false,
			trailing_semicolon: false,
			collapse_blank_lines: false,
			max_blank_lines: 2,
		}
	}
}

impl FormatConfig {
	#[cfg(feature = "cli")]
	pub fn load_from_dir(dir: &std::path::Path) -> Self {
		let config_path = dir.join(".surqlformat.toml");
		if let Ok(content) = std::fs::read_to_string(&config_path) {
			match toml::from_str::<FormatConfig>(&content) {
				Ok(config) => return config,
				Err(e) => {
					eprintln!(
						"Invalid .surqlformat.toml at {}: {e}",
						config_path.display()
					);
				}
			}
		}
		Self::default()
	}

	fn indent_str(&self) -> String {
		match self.indent_style {
			IndentStyle::Tab => "\t".to_string(),
			IndentStyle::Space => " ".repeat(self.indent_width as usize),
		}
	}
}

/// Format SurrealQL source text using the given config.
///
/// Returns `Some(formatted)` if changes were made, `None` if the source is already formatted.
pub fn format_source(source: &str, config: &FormatConfig) -> Option<String> {
	configurable_format(source, config)
}

/// Configurable lexer-based formatter.
fn configurable_format(source: &str, config: &FormatConfig) -> Option<String> {
	use crate::upstream::syn::lexer::Lexer;
	use crate::upstream::syn::token::{Delim, TokenKind};

	let bytes = source.as_bytes();
	if bytes.is_empty() || bytes.len() > u32::MAX as usize {
		return None;
	}

	let tokens: Vec<_> = match std::panic::catch_unwind(|| Lexer::new(bytes).collect()) {
		Ok(t) => t,
		Err(e) => {
			tracing::error!("Lexer panicked during formatting: {e:?}");
			return None;
		}
	};

	let mut result = String::with_capacity(source.len());
	let mut last_end: usize = 0;
	let mut depth: u32 = 0;
	let indent = config.indent_str();

	for token in &tokens {
		let start = token.span.offset as usize;
		let end = start + token.span.len as usize;
		if end > source.len() {
			continue;
		}

		let gap = &source[last_end..start];
		let original = &source[start..end];

		let formatted_token =
			if config.uppercase_keywords && matches!(token.kind, TokenKind::Keyword(_)) {
				let upper = original.to_uppercase();
				if is_formattable_keyword(&upper) {
					upper
				} else {
					original.to_string()
				}
			} else {
				original.to_string()
			};

		if token.kind == TokenKind::OpenDelim(Delim::Brace) {
			result.push_str(gap);
			result.push_str(&formatted_token);
			depth += 1;
			last_end = end;
			continue;
		}

		if token.kind == TokenKind::CloseDelim(Delim::Brace) {
			depth = depth.saturating_sub(1);
			result.push_str(gap);
			result.push_str(&formatted_token);
			last_end = end;
			continue;
		}

		if token.kind == TokenKind::SemiColon && config.newline_after_semicolon {
			result.push_str(gap);
			result.push_str(&formatted_token);

			let rest = &source[end..];
			let next_non_ws = rest.find(|c: char| c != ' ' && c != '\t');
			let already_newline = match next_non_ws {
				Some(pos) => rest.as_bytes()[pos] == b'\n',
				None => true,
			};

			if !already_newline {
				result.push('\n');
				for _ in 0..depth {
					result.push_str(&indent);
				}
			}

			last_end = end;
			continue;
		}

		if matches!(token.kind, TokenKind::Keyword(_)) {
			let upper = formatted_token.to_uppercase();
			let should_newline = (config.newline_before_where && upper == "WHERE")
				|| (config.newline_before_set && upper == "SET")
				|| (config.newline_before_from && upper == "FROM");

			if should_newline {
				let preceding = &source[..start];
				let last_newline = preceding.rfind('\n');
				let line_before = match last_newline {
					Some(pos) => &preceding[pos + 1..],
					None => preceding,
				};
				let already_on_new_line = line_before.trim().is_empty();

				if !already_on_new_line {
					result.push('\n');
					for _ in 0..depth.saturating_add(1) {
						result.push_str(&indent);
					}
					result.push_str(&formatted_token);
					last_end = end;
					continue;
				}
			}
		}

		if config.collapse_blank_lines && gap.contains('\n') {
			let newline_count = gap.matches('\n').count();
			if newline_count > (config.max_blank_lines as usize + 1) {
				let mut collapsed = String::new();
				let mut seen_newlines = 0u32;
				for ch in gap.chars() {
					if ch == '\n' {
						seen_newlines += 1;
						if seen_newlines <= config.max_blank_lines + 1 {
							collapsed.push(ch);
						}
					} else if seen_newlines <= config.max_blank_lines + 1 {
						collapsed.push(ch);
					}
				}
				result.push_str(&collapsed);
				result.push_str(&formatted_token);
				last_end = end;
				continue;
			}
		}

		result.push_str(gap);
		result.push_str(&formatted_token);
		last_end = end;
	}

	if last_end < source.len() {
		let trailing = &source[last_end..];

		if config.trailing_semicolon {
			let trimmed = trailing.trim_end();
			if !trimmed.is_empty() && !trimmed.ends_with(';') {
				result.push_str(trimmed);
				result.push(';');
				let ws_start = trimmed.len();
				if ws_start < trailing.len() {
					result.push_str(&trailing[ws_start..]);
				}
			} else {
				result.push_str(trailing);
			}
		} else {
			result.push_str(trailing);
		}
	} else if config.trailing_semicolon && !result.is_empty() && !result.trim_end().ends_with(';') {
		result.push(';');
	}

	if result == source { None } else { Some(result) }
}

fn is_formattable_keyword(upper: &str) -> bool {
	matches!(
		upper,
		"SELECT"
			| "FROM" | "WHERE"
			| "AND" | "OR"
			| "NOT" | "IN"
			| "ORDER" | "BY"
			| "GROUP" | "LIMIT"
			| "OFFSET"
			| "FETCH" | "CREATE"
			| "UPDATE"
			| "DELETE"
			| "INSERT"
			| "INTO" | "SET"
			| "UPSERT"
			| "MERGE" | "CONTENT"
			| "RETURN"
			| "DEFINE"
			| "REMOVE"
			| "TABLE" | "FIELD"
			| "INDEX" | "EVENT"
			| "FUNCTION"
			| "PARAM" | "NAMESPACE"
			| "DATABASE"
			| "ANALYZER"
			| "ACCESS"
			| "SCHEMAFULL"
			| "SCHEMALESS"
			| "TYPE" | "DEFAULT"
			| "READONLY"
			| "FLEXIBLE"
			| "UNIQUE"
			| "FIELDS"
			| "ON" | "WHEN"
			| "THEN" | "ELSE"
			| "END" | "IF"
			| "FOR" | "LET"
			| "BEGIN" | "COMMIT"
			| "CANCEL"
			| "TRANSACTION"
			| "USE" | "NS"
			| "DB" | "AS"
			| "IS" | "LIKE"
			| "CONTAINS"
			| "CONTAINSALL"
			| "CONTAINSANY"
			| "CONTAINSNONE"
			| "INSIDE"
			| "OUTSIDE"
			| "INTERSECTS"
			| "ALLINSIDE"
			| "ANYINSIDE"
			| "NONEINSIDE"
			| "ASC" | "DESC"
			| "COLLATE"
			| "NUMERIC"
			| "COMMENT"
			| "PERMISSIONS"
			| "FULL" | "NONE"
			| "RELATE"
			| "ONLY" | "VALUE"
			| "VALUES"
			| "OVERWRITE"
			| "EXISTS"
			| "ASSERT"
			| "ENFORCED"
			| "DROP" | "CHANGEFEED"
			| "INCLUDE"
			| "ORIGINAL"
			| "LIVE" | "DIFF"
			| "KILL" | "SHOW"
			| "INFO" | "SLEEP"
			| "THROW" | "BREAK"
			| "CONTINUE"
			| "PARALLEL"
			| "TIMEOUT"
			| "EXPLAIN"
			| "SPLIT" | "AT"
			| "TOKENIZERS"
			| "FILTERS"
			| "WITH" | "NOINDEX"
			| "UNIQ" | "SEARCH"
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_uppercase_keywords_with_default_config() {
		let source = "select * from user where age > 18;";
		let formatted = format_source(source, &FormatConfig::default()).unwrap();
		assert_eq!(formatted, "SELECT * FROM user WHERE age > 18;");
	}

	#[test]
	fn should_preserve_comments() {
		let source = "-- this is a comment\nselect * from user;";
		let formatted = format_source(source, &FormatConfig::default()).unwrap();
		assert!(formatted.contains("-- this is a comment"));
		assert!(formatted.contains("SELECT"));
	}

	#[test]
	fn should_preserve_strings() {
		let source = "select * from user where name = 'select from where';";
		let formatted = format_source(source, &FormatConfig::default()).unwrap();
		assert!(formatted.contains("'select from where'"));
	}

	#[test]
	fn should_return_none_when_already_formatted() {
		let source = "SELECT * FROM user WHERE age > 18;";
		let result = format_source(source, &FormatConfig::default());
		assert!(result.is_none(), "already formatted should return None");
	}

	#[test]
	fn should_add_newline_after_semicolon() {
		let config = FormatConfig {
			newline_after_semicolon: true,
			..Default::default()
		};
		let source = "DEFINE TABLE user; DEFINE TABLE post;";
		let formatted = format_source(source, &config).unwrap();
		assert!(
			formatted.contains("user;\n"),
			"should have newline after first semicolon: {formatted}"
		);
	}

	#[test]
	fn should_not_double_newline_after_semicolon() {
		let config = FormatConfig {
			newline_after_semicolon: true,
			..Default::default()
		};
		let source = "DEFINE TABLE user;\nDEFINE TABLE post;";
		let result = format_source(source, &config);
		assert!(result.is_none(), "already has newlines, should return None");
	}

	#[test]
	fn should_add_newline_before_where() {
		let config = FormatConfig {
			newline_before_where: true,
			..Default::default()
		};
		let source = "SELECT * FROM user WHERE age > 18;";
		let formatted = format_source(source, &config).unwrap();
		assert_eq!(
			formatted, "SELECT * FROM user\n\tWHERE age > 18;",
			"should have newline+indent before WHERE: {formatted}"
		);
	}

	#[test]
	fn should_collapse_blank_lines() {
		let config = FormatConfig {
			collapse_blank_lines: true,
			max_blank_lines: 1,
			..Default::default()
		};
		let source = "SELECT * FROM user;\n\n\n\n\nSELECT * FROM post;";
		let formatted = format_source(source, &config).unwrap();
		let newline_count = formatted.matches('\n').count();
		assert!(
			newline_count <= 3,
			"should collapse to max 1 blank line, got {newline_count} newlines: {formatted}"
		);
	}

	#[test]
	fn should_add_trailing_semicolon() {
		let config = FormatConfig {
			trailing_semicolon: true,
			uppercase_keywords: false,
			..Default::default()
		};
		let source = "SELECT * FROM user";
		let formatted = format_source(source, &config).unwrap();
		assert!(
			formatted.ends_with(';'),
			"should have trailing semicolon: {formatted}"
		);
	}

	#[test]
	fn should_apply_custom_config() {
		let config = FormatConfig {
			uppercase_keywords: true,
			indent_style: IndentStyle::Space,
			indent_width: 2,
			newline_after_semicolon: true,
			newline_before_where: true,
			..Default::default()
		};
		assert!(config.uppercase_keywords);
		assert_eq!(config.indent_style, IndentStyle::Space);
		assert_eq!(config.indent_width, 2);
		assert!(config.newline_after_semicolon);
		assert!(config.newline_before_where);
		assert!(!config.newline_before_set);
	}

	#[test]
	fn should_add_newline_before_set() {
		let config = FormatConfig {
			newline_before_set: true,
			..Default::default()
		};
		let source = "UPDATE user SET name = 'Alice';";
		let formatted = format_source(source, &config).unwrap();
		assert_eq!(
			formatted, "UPDATE user\n\tSET name = 'Alice';",
			"should have newline+indent before SET: {formatted}"
		);
	}
}
