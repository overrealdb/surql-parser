//! Declarative mappings loaded from mappings.toml

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Mappings {
	/// Import path rewrites: "crate::sql" → "crate::ast"
	pub import_rewrites: HashMap<String, String>,

	/// Code removal rules
	pub remove: RemoveRules,

	/// Attribute transformations
	#[serde(default)]
	pub attributes: AttributeRules,

	/// Warning configuration
	#[serde(default)]
	pub warnings: WarningConfig,
}

#[derive(Debug, Deserialize)]
pub struct RemoveRules {
	/// Parameter type fragments that indicate execution-only impl blocks.
	/// If any method in an impl block has a parameter containing one of these strings,
	/// the entire impl block is removed.
	#[serde(default)]
	pub execution_params: Vec<String>,

	/// Cross-layer From impl pairs to remove.
	/// Each entry is [from_module, to_module].
	/// Removes `impl From<from::X> for to::Y` and vice versa.
	#[serde(default)]
	pub cross_layer_from_impls: Vec<[String; 2]>,

	/// Module path prefixes to strip entirely (e.g., "crate::expr").
	/// Any `use` import from these modules is removed.
	/// Any impl block referencing these modules (in self_ty or trait_) is removed.
	#[serde(default)]
	pub strip_modules: Vec<String>,

	/// Module names to remove entirely (e.g., "test").
	#[serde(default)]
	pub remove_modules: Vec<String>,

	/// Cfg attributes to strip items for (e.g., "cfg(test)").
	#[serde(default)]
	#[allow(dead_code)]
	pub remove_cfg: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct AttributeRules {
	/// Attributes to remove entirely (e.g., "instrument").
	#[serde(default)]
	pub remove_attrs: Vec<String>,

	/// Attributes to wrap in cfg_attr.
	/// Key: attribute name (e.g., "instrument")
	/// Value: feature name (e.g., "tracing")
	#[serde(default)]
	#[allow(dead_code)]
	pub cfg_wrap: HashMap<String, String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct WarningConfig {
	/// Warn when encountering unknown crate:: paths not in import_rewrites.
	#[serde(default = "default_true")]
	#[allow(dead_code)]
	pub warn_unknown_imports: bool,

	/// Fail on unknown imports (strict mode).
	#[serde(default)]
	pub fail_on_unknown: bool,
}

fn default_true() -> bool {
	true
}

impl Mappings {
	pub fn load(path: &Path) -> Result<Self> {
		let content = std::fs::read_to_string(path)
			.with_context(|| format!("Failed to read {}", path.display()))?;
		let mappings: Mappings = toml::from_str(&content)
			.with_context(|| format!("Failed to parse {}", path.display()))?;
		Ok(mappings)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_example_mappings() {
		let toml_str = r#"
[import_rewrites]
"crate::sql" = "crate::ast"
"crate::syn" = "crate::parser"

[remove]
execution_params = ["Context", "Options"]
cross_layer_from_impls = [["sql", "expr"]]
remove_modules = ["test"]
remove_cfg = ["cfg(test)"]

[attributes.cfg_wrap]
"instrument" = "tracing"

[warnings]
warn_unknown_imports = true
fail_on_unknown = false
"#;

		let mappings: Mappings = toml::from_str(toml_str).unwrap();
		assert_eq!(mappings.import_rewrites.len(), 2);
		assert_eq!(mappings.remove.execution_params.len(), 2);
		assert_eq!(mappings.remove.cross_layer_from_impls.len(), 1);
		assert!(mappings.warnings.warn_unknown_imports);
		assert!(!mappings.warnings.fail_on_unknown);
	}
}
