use rmcp::model::{CallToolResult, Content};
use std::path::{Path, PathBuf};

pub(crate) fn error_result(msg: String) -> Result<CallToolResult, rmcp::ErrorData> {
	Ok(CallToolResult::error(vec![Content::text(msg)]))
}

pub(crate) fn validate_path_against(path: &str, allowed_root: &Path) -> Result<PathBuf, String> {
	let resolved =
		std::fs::canonicalize(path).map_err(|e| format!("Invalid path '{}': {e}", path))?;
	let root = std::fs::canonicalize(allowed_root)
		.map_err(|e| format!("Cannot resolve workspace root: {e}"))?;
	if !resolved.starts_with(&root) {
		return Err(format!(
			"Access denied: '{}' is outside working directory '{}'",
			resolved.display(),
			root.display()
		));
	}
	Ok(resolved)
}

pub(crate) fn is_valid_surql_identifier(s: &str) -> bool {
	!s.is_empty()
		&& s.len() <= 256
		&& s.chars()
			.all(|c| c.is_alphanumeric() || c == '_' || c == '-')
		&& s.chars()
			.next()
			.is_some_and(|c| c.is_alphabetic() || c == '_')
}
