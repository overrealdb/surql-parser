use std::path::Path;
use walkdir::WalkDir;

use crate::Error;
use crate::manifest::{Manifest, ModuleConfig};

/// A loaded schema module with its combined SQL content.
#[derive(Debug, Clone)]
pub struct SchemaModule {
	pub name: String,
	pub content: String,
	/// Relative paths of included .surql files.
	pub files: Vec<String>,
}

/// Load schema modules in dependency order (topological sort).
pub fn load_schema_modules(manifest: &Manifest) -> crate::Result<Vec<SchemaModule>> {
	let ordered = topological_sort(&manifest.modules)?;

	let root = manifest.root_path()?;
	let mut modules = Vec::new();
	for config in &ordered {
		let module = load_module(root, config)?;
		modules.push(module);
	}

	Ok(modules)
}

/// Extract function names from all schema modules using surql-parser.
pub fn extract_function_names(modules: &[SchemaModule]) -> crate::Result<Vec<String>> {
	let mut all_sql = String::new();
	for m in modules {
		all_sql.push_str(&m.content);
		all_sql.push('\n');
	}

	if all_sql.trim().is_empty() {
		return Ok(Vec::new());
	}

	surql_parser::list_functions(&all_sql)
		.map_err(|e| Error::Schema(format!("failed to extract function names: {e}")))
}

/// Load a single module: read all .surql files in the module's directory.
fn load_module(root: &Path, config: &ModuleConfig) -> crate::Result<SchemaModule> {
	let module_path = root.join(&config.path);
	if !module_path.exists() {
		return Err(Error::Schema(format!(
			"module path does not exist: {}",
			module_path.display(),
		)));
	}

	let mut content = String::new();
	let mut files = Vec::new();

	for entry in WalkDir::new(&module_path)
		.sort_by_file_name()
		.into_iter()
		.filter_map(|e| e.ok())
	{
		let path = entry.path();
		if path.extension().is_some_and(|ext| ext == "surql") {
			let sql = std::fs::read_to_string(path)
				.map_err(|e| Error::Schema(format!("failed to read {}: {e}", path.display())))?;

			if !content.is_empty() {
				content.push('\n');
			}
			content.push_str(&sql);

			let rel = path
				.strip_prefix(root)
				.unwrap_or(path)
				.to_string_lossy()
				.to_string();
			files.push(rel);
		}
	}

	if content.is_empty() {
		return Err(Error::Schema(format!(
			"module '{}' has no .surql files in {}",
			config.name,
			module_path.display(),
		)));
	}

	Ok(SchemaModule {
		name: config.name.clone(),
		content,
		files,
	})
}

/// Topological sort of modules by `depends_on` (Kahn's algorithm).
pub(crate) fn topological_sort(modules: &[ModuleConfig]) -> crate::Result<Vec<ModuleConfig>> {
	use std::collections::{HashMap, VecDeque};

	if modules.is_empty() {
		return Ok(Vec::new());
	}

	let module_map: HashMap<&str, &ModuleConfig> =
		modules.iter().map(|m| (m.name.as_str(), m)).collect();

	// Validate all dependencies exist
	for module in modules {
		for dep in &module.depends_on {
			if !module_map.contains_key(dep.as_str()) {
				return Err(Error::Schema(format!(
					"module '{}' depends on unknown module '{dep}'",
					module.name,
				)));
			}
		}
	}

	// Build in-degree map and adjacency list
	let mut in_degree: HashMap<&str, usize> = HashMap::new();
	let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

	for module in modules {
		in_degree.entry(module.name.as_str()).or_insert(0);
		for dep in &module.depends_on {
			*in_degree.entry(module.name.as_str()).or_insert(0) += 1;
			dependents
				.entry(dep.as_str())
				.or_default()
				.push(module.name.as_str());
		}
	}

	// Start with nodes that have no dependencies
	let mut ready: Vec<&str> = in_degree
		.iter()
		.filter(|(_, deg)| **deg == 0)
		.map(|(name, _)| *name)
		.collect();
	ready.sort(); // deterministic output

	let mut queue: VecDeque<&str> = ready.into_iter().collect();
	let mut result = Vec::new();

	while let Some(name) = queue.pop_front() {
		result.push(module_map[name].clone());

		if let Some(deps) = dependents.get(name) {
			let mut next = Vec::new();
			for &dep in deps {
				// All deps validated to exist at lines 110-118; in_degree populated from the same set
				let degree = in_degree.get_mut(dep).unwrap();
				*degree -= 1;
				if *degree == 0 {
					next.push(dep);
				}
			}
			next.sort();
			queue.extend(next);
		}
	}

	if result.len() != modules.len() {
		return Err(Error::Schema(
			"circular dependency detected in schema modules".into(),
		));
	}

	Ok(result)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn module(name: &str, deps: &[&str]) -> ModuleConfig {
		ModuleConfig {
			name: name.into(),
			path: format!("schema/{name}"),
			depends_on: deps.iter().map(|s| s.to_string()).collect(),
		}
	}

	#[test]
	fn topo_sort_empty() {
		let result = topological_sort(&[]).unwrap();
		assert!(result.is_empty());
	}

	#[test]
	fn topo_sort_no_deps() {
		let modules = vec![module("b", &[]), module("a", &[])];
		let result = topological_sort(&modules).unwrap();
		// Alphabetical order when no deps
		assert_eq!(result[0].name, "a");
		assert_eq!(result[1].name, "b");
	}

	#[test]
	fn topo_sort_linear_chain() {
		let modules = vec![module("c", &["b"]), module("a", &[]), module("b", &["a"])];
		let result = topological_sort(&modules).unwrap();
		assert_eq!(result[0].name, "a");
		assert_eq!(result[1].name, "b");
		assert_eq!(result[2].name, "c");
	}

	#[test]
	fn topo_sort_diamond() {
		let modules = vec![
			module("d", &["b", "c"]),
			module("a", &[]),
			module("b", &["a"]),
			module("c", &["a"]),
		];
		let result = topological_sort(&modules).unwrap();
		assert_eq!(result[0].name, "a");
		// b and c can be in either order, but both before d
		assert!(result[1].name == "b" || result[1].name == "c");
		assert!(result[2].name == "b" || result[2].name == "c");
		assert_eq!(result[3].name, "d");
	}

	#[test]
	fn topo_sort_rejects_cycle() {
		let modules = vec![module("a", &["b"]), module("b", &["a"])];
		assert!(topological_sort(&modules).is_err());
	}

	#[test]
	fn topo_sort_rejects_unknown_dep() {
		let modules = vec![module("a", &["nonexistent"])];
		assert!(topological_sort(&modules).is_err());
	}

	#[test]
	fn topo_sort_self_dependency() {
		let modules = vec![module("a", &["a"])];
		assert!(topological_sort(&modules).is_err());
	}

	#[test]
	fn topo_sort_three_node_cycle() {
		let modules = vec![
			module("a", &["c"]),
			module("b", &["a"]),
			module("c", &["b"]),
		];
		assert!(topological_sort(&modules).is_err());
	}

	#[test]
	fn topo_sort_single_module() {
		let modules = vec![module("only", &[])];
		let result = topological_sort(&modules).unwrap();
		assert_eq!(result.len(), 1);
		assert_eq!(result[0].name, "only");
	}

	#[test]
	fn topo_sort_wide_fan_out() {
		// a has no deps, b/c/d/e all depend on a
		let modules = vec![
			module("e", &["a"]),
			module("d", &["a"]),
			module("c", &["a"]),
			module("b", &["a"]),
			module("a", &[]),
		];
		let result = topological_sort(&modules).unwrap();
		assert_eq!(result[0].name, "a");
		// b, c, d, e in alphabetical order
		assert_eq!(result[1].name, "b");
		assert_eq!(result[2].name, "c");
		assert_eq!(result[3].name, "d");
		assert_eq!(result[4].name, "e");
	}

	#[test]
	fn topo_sort_wide_fan_in() {
		// a, b, c have no deps; d depends on all three
		let modules = vec![
			module("d", &["a", "b", "c"]),
			module("c", &[]),
			module("b", &[]),
			module("a", &[]),
		];
		let result = topological_sort(&modules).unwrap();
		// a, b, c first (alphabetical), then d
		assert_eq!(result[0].name, "a");
		assert_eq!(result[1].name, "b");
		assert_eq!(result[2].name, "c");
		assert_eq!(result[3].name, "d");
	}

	#[test]
	fn topo_sort_preserves_path() {
		let modules = vec![ModuleConfig {
			name: "core".into(),
			path: "custom/path/to/core".into(),
			depends_on: vec![],
		}];
		let result = topological_sort(&modules).unwrap();
		assert_eq!(result[0].path, "custom/path/to/core");
	}

	#[test]
	fn topo_sort_complex_graph() {
		// Real-world-ish: _shared → entity → search, _shared → events → analytics
		let modules = vec![
			module("analytics", &["events"]),
			module("search", &["entity"]),
			module("events", &["_shared"]),
			module("entity", &["_shared"]),
			module("_shared", &[]),
		];
		let result = topological_sort(&modules).unwrap();
		let names: Vec<&str> = result.iter().map(|m| m.name.as_str()).collect();
		// _shared must be first
		assert_eq!(names[0], "_shared");
		// entity before search, events before analytics
		assert!(
			names.iter().position(|&n| n == "entity").unwrap()
				< names.iter().position(|&n| n == "search").unwrap()
		);
		assert!(
			names.iter().position(|&n| n == "events").unwrap()
				< names.iter().position(|&n| n == "analytics").unwrap()
		);
	}
}
