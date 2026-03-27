use std::path::{Path, PathBuf};

/// Categorized file lists for smart project loading.
#[derive(Debug, Default)]
pub(crate) struct CategorizedFiles {
	pub schema: Vec<PathBuf>,
	pub functions: Vec<PathBuf>,
	pub migrations: Vec<PathBuf>,
	pub examples: Vec<PathBuf>,
	pub general: Vec<PathBuf>,
}

/// Classify each file by its parent directory or file name.
pub(crate) fn categorize_files(files: &[PathBuf]) -> CategorizedFiles {
	let mut result = CategorizedFiles::default();

	for path in files {
		let category = classify_file(path);
		match category {
			FileCategory::Schema => result.schema.push(path.clone()),
			FileCategory::Function => result.functions.push(path.clone()),
			FileCategory::Migration => result.migrations.push(path.clone()),
			FileCategory::Example => result.examples.push(path.clone()),
			FileCategory::General => result.general.push(path.clone()),
		}
	}

	result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileCategory {
	Schema,
	Function,
	Migration,
	Example,
	General,
}

pub(crate) fn classify_file(path: &Path) -> FileCategory {
	let parent_names: Vec<String> = path
		.ancestors()
		.filter_map(|a| a.file_name())
		.map(|n| n.to_string_lossy().to_lowercase())
		.collect();

	let in_dir = |name: &str| parent_names.iter().any(|d| d.contains(name));

	let file_stem = path
		.file_stem()
		.and_then(|n| n.to_str())
		.unwrap_or("")
		.to_lowercase();

	if in_dir("example") || in_dir("seed") || in_dir("test") {
		FileCategory::Example
	} else if in_dir("schema") || file_stem.starts_with("schema") {
		FileCategory::Schema
	} else if in_dir("function") || file_stem.starts_with("function") {
		FileCategory::Function
	} else if in_dir("migration") {
		FileCategory::Migration
	} else {
		FileCategory::General
	}
}

/// Inject OVERWRITE after DEFINE keywords so schema files are idempotent.
///
/// Transforms `DEFINE TABLE foo` into `DEFINE TABLE OVERWRITE foo`, etc.
/// Skips lines that already contain OVERWRITE.
pub fn inject_overwrite(content: &str) -> String {
	const DEFINE_KEYWORDS: &[&str] = &[
		"DEFINE TABLE",
		"DEFINE FIELD",
		"DEFINE INDEX",
		"DEFINE FUNCTION",
		"DEFINE EVENT",
		"DEFINE ANALYZER",
		"DEFINE PARAM",
	];

	let mut result = String::with_capacity(content.len() + 128);
	let mut in_block_comment = false;
	for line in content.lines() {
		let trimmed = line.trim_start();

		// Limitation: only detects block comments that start at the beginning of the
		// (trimmed) line. Mid-line `/* ... */` on a DEFINE line is not detected, so
		// OVERWRITE would still be injected. This is acceptable — mid-line block
		// comments before a DEFINE keyword are extremely rare in practice.
		if trimmed.starts_with("/*") {
			in_block_comment = true;
		}
		if in_block_comment || trimmed.starts_with("--") {
			result.push_str(line);
			result.push('\n');
			if trimmed.contains("*/") {
				in_block_comment = false;
			}
			continue;
		}

		let upper = trimmed.to_uppercase();
		let mut replaced = false;
		for keyword in DEFINE_KEYWORDS {
			if upper.starts_with(keyword) {
				let after_keyword = &trimmed[keyword.len()..];
				let after_upper = after_keyword.trim_start().to_uppercase();
				if after_upper.starts_with("OVERWRITE") || after_upper.starts_with("IF NOT EXISTS")
				{
					break;
				}
				let leading_ws = &line[..line.len() - line.trim_start().len()];
				result.push_str(leading_ws);
				result.push_str(&trimmed[..keyword.len()]);
				result.push_str(" OVERWRITE");
				result.push_str(after_keyword);
				result.push('\n');
				replaced = true;
				break;
			}
		}
		if !replaced {
			result.push_str(line);
			result.push('\n');
		}
	}
	if !content.ends_with('\n') && result.ends_with('\n') {
		result.pop();
	}
	result
}
