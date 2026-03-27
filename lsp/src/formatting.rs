//! LSP formatting layer.
//!
//! Delegates to the shared `surql_parser::formatting` module for the actual
//! formatting logic. This module handles the LSP protocol wrapping (TextEdit).

use tower_lsp::lsp_types::{Position, Range, TextEdit};

pub use surql_parser::formatting::{FormatConfig, IndentStyle};

pub fn load_config_from_workspace(dir: &std::path::Path) -> FormatConfig {
	let config_path = dir.join(".surqlformat.toml");
	if let Ok(content) = std::fs::read_to_string(&config_path) {
		match toml::from_str::<FormatConfig>(&content) {
			Ok(config) => {
				tracing::info!("Loaded format config from {}", config_path.display());
				return config;
			}
			Err(e) => {
				tracing::warn!(
					"Invalid .surqlformat.toml at {}: {e}",
					config_path.display()
				);
			}
		}
	}
	FormatConfig::default()
}

#[cfg(test)]
pub fn format_document_for_test(source: &str) -> Option<Vec<TextEdit>> {
	format_document(source, &FormatConfig::default())
}

pub fn format_document(source: &str, config: &FormatConfig) -> Option<Vec<TextEdit>> {
	if cfg!(feature = "canonical-format") {
		canonical_format(source)
	} else {
		configurable_format(source, config)
	}
}

fn canonical_format(source: &str) -> Option<Vec<TextEdit>> {
	let ast = surql_parser::parse(source).ok()?;
	let formatted = surql_parser::format(&ast);
	if formatted == source {
		return None;
	}
	Some(vec![full_document_edit(source, formatted)])
}

fn configurable_format(source: &str, config: &FormatConfig) -> Option<Vec<TextEdit>> {
	let formatted = surql_parser::formatting::format_source(source, config)?;
	Some(vec![full_document_edit(source, formatted)])
}

fn full_document_edit(source: &str, new_text: String) -> TextEdit {
	let lines: Vec<&str> = source.split('\n').collect();
	let last_line_idx = lines.len().saturating_sub(1) as u32;
	let last_line_chars = lines
		.last()
		.map(|l| {
			l.strip_suffix('\r')
				.unwrap_or(l)
				.chars()
				.map(|c| c.len_utf16())
				.sum::<usize>()
		})
		.unwrap_or(0) as u32;
	TextEdit {
		range: Range {
			start: Position {
				line: 0,
				character: 0,
			},
			end: Position {
				line: last_line_idx,
				character: last_line_chars,
			},
		},
		new_text,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_uppercase_keywords_with_default_config() {
		let source = "select * from user where age > 18;";
		let edits = format_document(source, &FormatConfig::default()).unwrap();
		assert_eq!(edits.len(), 1);
		let formatted = &edits[0].new_text;
		assert!(formatted.contains("SELECT"));
		assert!(formatted.contains("FROM"));
		assert!(formatted.contains("WHERE"));
		assert!(formatted.contains("age > 18"));
	}

	#[test]
	fn should_preserve_comments() {
		let source = "-- this is a comment\nselect * from user;";
		let edits = format_document(source, &FormatConfig::default()).unwrap();
		let formatted = &edits[0].new_text;
		assert!(formatted.contains("-- this is a comment"));
		assert!(formatted.contains("SELECT"));
	}

	#[test]
	fn should_preserve_strings() {
		let source = "select * from user where name = 'select from where';";
		let edits = format_document(source, &FormatConfig::default()).unwrap();
		let formatted = &edits[0].new_text;
		assert!(formatted.contains("'select from where'"));
	}

	#[test]
	fn should_return_none_when_nothing_to_change() {
		let source = "SELECT * FROM user WHERE age > 18;";
		let result = format_document(source, &FormatConfig::default());
		assert!(result.is_none(), "already formatted should return None");
	}

	#[test]
	fn should_add_newline_after_semicolon() {
		let config = FormatConfig {
			newline_after_semicolon: true,
			..Default::default()
		};
		let source = "DEFINE TABLE user; DEFINE TABLE post;";
		let edits = format_document(source, &config).unwrap();
		let formatted = &edits[0].new_text;
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
		let result = format_document(source, &config);
		assert!(result.is_none(), "already has newlines, should return None");
	}

	#[test]
	fn should_add_newline_before_where() {
		let config = FormatConfig {
			newline_before_where: true,
			..Default::default()
		};
		let source = "SELECT * FROM user WHERE age > 18;";
		let edits = format_document(source, &config).unwrap();
		let formatted = &edits[0].new_text;
		assert!(
			formatted.contains("\n") && formatted.contains("WHERE"),
			"should have newline before WHERE: {formatted}"
		);
	}

	#[test]
	fn should_add_newline_before_set() {
		let config = FormatConfig {
			newline_before_set: true,
			..Default::default()
		};
		let source = "UPDATE user SET name = 'Alice';";
		let edits = format_document(source, &config).unwrap();
		let formatted = &edits[0].new_text;
		assert!(
			formatted.contains("\n"),
			"should have newline before SET: {formatted}"
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
		let edits = format_document(source, &config).unwrap();
		let formatted = &edits[0].new_text;
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
		let edits = format_document(source, &config).unwrap();
		let formatted = &edits[0].new_text;
		assert!(
			formatted.ends_with(';'),
			"should have trailing semicolon: {formatted}"
		);
	}

	#[test]
	fn should_not_double_trailing_semicolon() {
		let config = FormatConfig {
			trailing_semicolon: true,
			uppercase_keywords: false,
			..Default::default()
		};
		let source = "SELECT * FROM user;";
		let result = format_document(source, &config);
		assert!(result.is_none(), "already has semicolon");
	}

	#[test]
	fn should_deserialize_config_from_toml() {
		let toml_str = r#"
uppercase_keywords = true
indent_style = "space"
indent_width = 2
newline_after_semicolon = true
newline_before_where = true
"#;
		let config: FormatConfig = toml::from_str(toml_str).unwrap();
		assert!(config.uppercase_keywords);
		assert_eq!(config.indent_style, IndentStyle::Space);
		assert_eq!(config.indent_width, 2);
		assert!(config.newline_after_semicolon);
		assert!(config.newline_before_where);
		assert!(!config.newline_before_set);
	}

	#[test]
	fn should_use_defaults_for_missing_fields() {
		let toml_str = "uppercase_keywords = false\n";
		let config: FormatConfig = toml::from_str(toml_str).unwrap();
		assert!(!config.uppercase_keywords);
		assert_eq!(config.indent_style, IndentStyle::Tab);
		assert_eq!(config.indent_width, 4);
		assert!(!config.newline_after_semicolon);
	}

	#[test]
	fn should_combine_multiple_rules() {
		let config = FormatConfig {
			newline_after_semicolon: true,
			newline_before_where: true,
			..Default::default()
		};
		let source = "select * from user where age > 18; select * from post;";
		let edits = format_document(source, &config).unwrap();
		let formatted = &edits[0].new_text;
		assert!(formatted.contains("SELECT"));
		assert!(formatted.contains("FROM"));
		assert!(formatted.contains("\n"));
	}
}
