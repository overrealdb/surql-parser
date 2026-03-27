/// Maximum `.surql` file size (10 MB). Files larger than this are rejected to
/// prevent accidental memory exhaustion.
pub const MAX_SURQL_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Read a `.surql` file with a size guard.
///
/// Returns an error if the file is larger than [`MAX_SURQL_FILE_SIZE`] or
/// cannot be read for any other reason.
pub fn read_surql_file(path: &std::path::Path) -> Result<String, String> {
	let meta = std::fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
	if meta.len() > MAX_SURQL_FILE_SIZE {
		return Err(format!(
			"{}: file too large ({} bytes, max {})",
			path.display(),
			meta.len(),
			MAX_SURQL_FILE_SIZE
		));
	}
	std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Recursively collect all `.surql` files from a directory tree.
///
/// Skips known large or irrelevant directories: `target`, `node_modules`, `.git`,
/// `build`, `fixtures`, `dist`, `.cache`, `surql-lsp-out`, and any directory
/// whose name starts with `.`.
///
/// Collected paths are absolute. Callers that need relative paths can use
/// [`Path::strip_prefix`] on the results.
pub fn collect_surql_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
	let mut visited = std::collections::HashSet::new();
	if let Ok(canonical) = dir.canonicalize() {
		visited.insert(canonical);
	}
	collect_surql_files_recursive(dir, out, 0, &mut visited);
}

pub(crate) fn collect_surql_files_recursive(
	dir: &std::path::Path,
	out: &mut Vec<std::path::PathBuf>,
	depth: u32,
	visited: &mut std::collections::HashSet<std::path::PathBuf>,
) {
	if depth > 32 {
		warn!(
			"Max directory depth (32) exceeded at {}, skipping",
			dir.display()
		);
		return;
	}
	let entries = match std::fs::read_dir(dir) {
		Ok(e) => e,
		Err(e) => {
			warn!("Cannot read directory {}: {e}", dir.display());
			return;
		}
	};
	for entry in entries {
		let entry = match entry {
			Ok(e) => e,
			Err(e) => {
				warn!("Skipping unreadable entry in {}: {e}", dir.display());
				continue;
			}
		};
		let path = entry.path();
		if path
			.symlink_metadata()
			.map(|m| m.is_symlink())
			.unwrap_or(false)
		{
			warn!("Skipping symlink: {}", path.display());
			continue;
		}
		if path.is_dir() {
			let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
			if matches!(
				name,
				"target"
					| "node_modules"
					| ".git" | "build"
					| "fixtures" | "dist"
					| ".cache" | "surql-lsp-out"
			) || name.starts_with('.')
			{
				continue;
			}
			if let Ok(canonical) = path.canonicalize()
				&& !visited.insert(canonical)
			{
				warn!(
					"Skipping already-visited directory (symlink cycle?): {}",
					path.display()
				);
				continue;
			}
			collect_surql_files_recursive(&path, out, depth + 1, visited);
		} else if path.extension().is_some_and(|ext| ext == "surql") {
			out.push(path);
		}
	}
}
