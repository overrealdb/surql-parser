use std::path::PathBuf;

/// Load SurrealDB connection scope from a `.env` file in the workspace root.
///
/// Recognizes `SURREALDB_URL`/`SURREAL_URL`, `SURREALDB_NS`/`SURREAL_NS`,
/// and `SURREALDB_DB`/`SURREAL_DB`. Returns `(url, ns, db)` only when both
/// NS and DB are present.
pub(crate) fn load_dotenv(root: &std::path::Path) -> Option<(String, String, String)> {
	let files = [".env", ".env.development", ".env.local"];
	let mut url = None;
	let mut ns = None;
	let mut db = None;
	let mut found_any = false;
	for name in &files {
		if let Ok(content) = std::fs::read_to_string(root.join(name)) {
			found_any = true;
			for line in content.lines() {
				let line = line.trim();
				if line.starts_with('#') || line.is_empty() {
					continue;
				}
				if let Some((key, value)) = line.split_once('=') {
					let key = key.trim();
					let value = value.trim().trim_matches('"').trim_matches('\'');
					match key {
						"SURREALDB_URL" | "SURREAL_URL" => url = Some(value.to_string()),
						"SURREALDB_NS" | "SURREAL_NS" => ns = Some(value.to_string()),
						"SURREALDB_DB" | "SURREAL_DB" => db = Some(value.to_string()),
						_ => {}
					}
				}
			}
		}
	}
	if !found_any {
		return None;
	}
	let ns = ns?;
	let db = db?;
	let url = url.unwrap_or_else(|| {
		tracing::debug!(".env has NS/DB but no URL — using empty URL");
		String::new()
	});
	Some((url, ns, db))
}

/// Scan the workspace for subdirectories that each contain their own SurrealDB
/// project marker (`manifest.toml`, `.surqlformat.toml`, or `.env` with
/// `SURREALDB_*` vars). Returns the list of detected project roots.
///
/// Walks one level of subdirectories to avoid deep filesystem traversal.
/// Stops at workspace root (never walks above it).
pub(crate) fn detect_monorepo_projects(root: &std::path::Path) -> Vec<PathBuf> {
	let mut projects = Vec::new();

	// Check the workspace root itself
	if has_surql_project_marker(root) {
		projects.push(root.to_path_buf());
	}

	// Walk immediate subdirectories
	let Ok(entries) = std::fs::read_dir(root) else {
		return projects;
	};
	for entry in entries.flatten() {
		let path = entry.path();
		if !path.is_dir() {
			continue;
		}
		// Skip hidden dirs and output dirs
		let name = entry.file_name();
		let name_str = name.to_string_lossy();
		if name_str.starts_with('.') || name_str == "target" || name_str == "node_modules" {
			continue;
		}
		if has_surql_project_marker(&path) {
			projects.push(path.clone());
			// Already detected as a project — don't scan its children
			continue;
		}
		// Not a project itself: check one level deeper (e.g. services/auth/)
		if let Ok(sub_entries) = std::fs::read_dir(&path) {
			for sub_entry in sub_entries.flatten() {
				let sub_path = sub_entry.path();
				if sub_path.is_dir() {
					let sub_name = sub_entry.file_name();
					let sub_name_str = sub_name.to_string_lossy();
					if !sub_name_str.starts_with('.')
						&& sub_name_str != "target"
						&& sub_name_str != "node_modules"
						&& has_surql_project_marker(&sub_path)
					{
						projects.push(sub_path);
					}
				}
			}
		}
	}

	projects
}

/// Check whether a directory contains a SurrealDB project marker:
/// - `manifest.toml` with `[meta]` section
/// - `.surqlformat.toml`
/// - `.env` with `SURREALDB_NS` or `SURREAL_NS`
pub(crate) fn has_surql_project_marker(dir: &std::path::Path) -> bool {
	// manifest.toml with [meta] section
	if let Ok(content) = std::fs::read_to_string(dir.join("manifest.toml"))
		&& content.contains("[meta]")
	{
		return true;
	}
	// Also check surql/manifest.toml and sql/manifest.toml
	for sub in ["surql", "sql"] {
		if let Ok(content) = std::fs::read_to_string(dir.join(sub).join("manifest.toml"))
			&& content.contains("[meta]")
		{
			return true;
		}
	}
	// .surqlformat.toml
	if dir.join(".surqlformat.toml").exists() {
		return true;
	}
	// .env / .env.local / .env.development with SurrealDB vars
	for env_name in [".env.local", ".env.development", ".env"] {
		if let Ok(content) = std::fs::read_to_string(dir.join(env_name))
			&& (content.contains("SURREALDB_NS") || content.contains("SURREAL_NS"))
		{
			return true;
		}
	}
	false
}
