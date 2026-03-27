use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::Error;
use crate::migration::{Migration, compute_checksum, validate_migration_sequence};
use crate::schema::SchemaModule;

/// Top-level metadata from `manifest.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct ManifestMeta {
	pub ns: String,
	pub db: String,
	pub system_db: String,
	#[serde(default)]
	pub surrealdb: Option<String>,
}

/// Configuration for a single schema module.
#[derive(Debug, Clone, Deserialize)]
pub struct ModuleConfig {
	pub name: String,
	pub path: String,
	#[serde(default)]
	pub depends_on: Vec<String>,
}

/// Parsed manifest with resolved root path.
///
/// Can be constructed in two ways:
/// - [`Manifest::load()`] — from a `manifest.toml` on disk (CLI / runtime)
/// - [`Manifest::builder()`] — programmatically with inline content (library / embedded)
#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
	pub meta: ManifestMeta,
	#[serde(default)]
	pub modules: Vec<ModuleConfig>,
	/// Root path for filesystem-based manifests. `None` for embedded manifests.
	#[serde(skip)]
	pub root: Option<PathBuf>,
	/// Pre-loaded migrations (embedded mode). When `Some`, `discover_migrations` is bypassed.
	#[serde(skip)]
	pub preloaded_migrations: Option<Vec<Migration>>,
	/// Pre-loaded schema modules (embedded mode). When `Some`, `load_schema_modules` is bypassed.
	#[serde(skip)]
	pub preloaded_modules: Option<Vec<SchemaModule>>,
}

impl Manifest {
	/// Load a manifest from `{root}/manifest.toml`.
	pub fn load(root: impl AsRef<Path>) -> crate::Result<Self> {
		let root = root.as_ref().to_path_buf();
		let manifest_path = root.join("manifest.toml");
		let content = std::fs::read_to_string(&manifest_path).map_err(|e| {
			Error::Manifest(format!("failed to read {}: {e}", manifest_path.display()))
		})?;
		let mut manifest: Manifest = toml::from_str(&content)?;
		manifest.root = Some(root);
		Ok(manifest)
	}

	/// Create a builder for programmatic manifest construction (embedded mode).
	pub fn builder() -> ManifestBuilder {
		ManifestBuilder::default()
	}

	/// Path to the root directory. Returns `Err` for embedded manifests.
	pub fn root_path(&self) -> crate::Result<&Path> {
		self.root
			.as_deref()
			.ok_or_else(|| Error::Manifest("no root path — this is an embedded manifest".into()))
	}

	/// Path to the migrations directory.
	pub fn migrations_dir(&self) -> crate::Result<PathBuf> {
		Ok(self.root_path()?.join("migrations"))
	}

	/// Path to the generated output directory.
	pub fn generated_dir(&self) -> crate::Result<PathBuf> {
		Ok(self.root_path()?.join("generated"))
	}
}

/// Builder for constructing a [`Manifest`] programmatically.
///
/// # Example
///
/// ```rust,ignore
/// let manifest = Manifest::builder()
///     .meta("my_ns", "main", "_system")
///     .migration(1, "seed", include_str!("../surql/migrations/v001_seed.surql"))
///     .module("_shared", &[], &[include_str!("../surql/schema/_shared/analyzers.surql")])
///     .module("entity", &["_shared"], &[
///         include_str!("../surql/schema/entity/table.surql"),
///         include_str!("../surql/schema/entity/fn.surql"),
///     ])
///     .build()?;
/// ```
#[derive(Debug, Default)]
pub struct ManifestBuilder {
	meta: Option<ManifestMeta>,
	migrations: Vec<Migration>,
	modules: Vec<(String, Vec<String>, Vec<String>)>, // (name, depends_on, content_parts)
}

impl ManifestBuilder {
	/// Set the manifest metadata.
	pub fn meta(mut self, ns: &str, db: &str, system_db: &str) -> Self {
		self.meta = Some(ManifestMeta {
			ns: ns.into(),
			db: db.into(),
			system_db: system_db.into(),
			surrealdb: None,
		});
		self
	}

	/// Set the manifest metadata with a full [`ManifestMeta`].
	pub fn meta_full(mut self, meta: ManifestMeta) -> Self {
		self.meta = Some(meta);
		self
	}

	/// Add a migration with version, name, and SQL content.
	pub fn migration(mut self, version: u32, name: &str, content: &str) -> Self {
		let checksum = compute_checksum(content);
		self.migrations.push(Migration {
			version,
			name: name.into(),
			content: content.into(),
			checksum,
			down_content: None,
		});
		self
	}

	/// Add a migration with both up and down SQL content.
	///
	/// The `down_content` is executed during rollback to reverse the migration.
	pub fn migration_with_down(
		mut self,
		version: u32,
		name: &str,
		content: &str,
		down_content: &str,
	) -> Self {
		let checksum = compute_checksum(content);
		self.migrations.push(Migration {
			version,
			name: name.into(),
			content: content.into(),
			checksum,
			down_content: Some(down_content.into()),
		});
		self
	}

	/// Add a schema module with dependencies and content parts.
	///
	/// Content parts are concatenated with newlines (like reading multiple .surql files).
	pub fn module(mut self, name: &str, depends_on: &[&str], content: &[&str]) -> Self {
		self.modules.push((
			name.into(),
			depends_on.iter().map(|s| s.to_string()).collect(),
			content.iter().map(|s| s.to_string()).collect(),
		));
		self
	}

	/// Build the manifest, validating migrations and sorting modules by dependency order.
	pub fn build(self) -> crate::Result<Manifest> {
		let meta = self
			.meta
			.ok_or_else(|| Error::Manifest("meta is required".into()))?;

		// Validate migration sequence
		let mut migrations = self.migrations;
		migrations.sort_by_key(|m| m.version);
		validate_migration_sequence(&migrations)?;

		// Build module configs for topological sort
		let module_configs: Vec<ModuleConfig> = self
			.modules
			.iter()
			.map(|(name, deps, _)| ModuleConfig {
				name: name.clone(),
				path: String::new(), // unused in embedded mode
				depends_on: deps.clone(),
			})
			.collect();

		// Sort modules by dependency order
		let ordered = crate::schema::topological_sort(&module_configs)?;

		// Build SchemaModules in sorted order
		let module_content: std::collections::HashMap<&str, &Vec<String>> = self
			.modules
			.iter()
			.map(|(name, _, content)| (name.as_str(), content))
			.collect();

		let schema_modules: Vec<SchemaModule> = ordered
			.iter()
			.map(|config| {
				let parts = module_content[config.name.as_str()];
				let content = parts.join("\n");
				SchemaModule {
					name: config.name.clone(),
					content,
					files: vec![],
				}
			})
			.collect();

		Ok(Manifest {
			meta,
			modules: module_configs,
			root: None,
			preloaded_migrations: Some(migrations),
			preloaded_modules: Some(schema_modules),
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_minimal_manifest() {
		let toml = r#"
			[meta]
			ns = "test"
			db = "main"
			system_db = "_system"
		"#;
		let manifest: Manifest = toml::from_str(toml).unwrap();
		assert_eq!(manifest.meta.ns, "test");
		assert_eq!(manifest.meta.db, "main");
		assert_eq!(manifest.meta.system_db, "_system");
		assert!(manifest.modules.is_empty());
		assert_eq!(manifest.meta.surrealdb, None);
	}

	#[test]
	fn parse_full_manifest() {
		let toml = r#"
			[meta]
			ns = "datacat"
			db = "main"
			system_db = "_system"
			surrealdb = ">=3.0.0"

			[[modules]]
			name = "_shared"
			path = "schema/_shared"

			[[modules]]
			name = "entity"
			path = "schema/entity"
			depends_on = ["_shared"]
		"#;
		let manifest: Manifest = toml::from_str(toml).unwrap();
		assert_eq!(manifest.meta.ns, "datacat");
		assert_eq!(manifest.meta.surrealdb, Some(">=3.0.0".into()));
		assert_eq!(manifest.modules.len(), 2);
		assert_eq!(manifest.modules[0].name, "_shared");
		assert_eq!(manifest.modules[0].path, "schema/_shared");
		assert!(manifest.modules[0].depends_on.is_empty());
		assert_eq!(manifest.modules[1].name, "entity");
		assert_eq!(manifest.modules[1].depends_on, vec!["_shared"]);
	}

	#[test]
	fn parse_rejects_missing_meta() {
		let toml = r#"
			[[modules]]
			name = "a"
			path = "schema/a"
		"#;
		assert!(toml::from_str::<Manifest>(toml).is_err());
	}

	#[test]
	fn parse_rejects_missing_ns() {
		let toml = r#"
			[meta]
			db = "main"
			system_db = "_system"
		"#;
		assert!(toml::from_str::<Manifest>(toml).is_err());
	}

	#[test]
	fn parse_rejects_missing_db() {
		let toml = r#"
			[meta]
			ns = "test"
			system_db = "_system"
		"#;
		assert!(toml::from_str::<Manifest>(toml).is_err());
	}

	#[test]
	fn parse_rejects_missing_system_db() {
		let toml = r#"
			[meta]
			ns = "test"
			db = "main"
		"#;
		assert!(toml::from_str::<Manifest>(toml).is_err());
	}

	#[test]
	fn parse_rejects_invalid_toml() {
		assert!(toml::from_str::<Manifest>("{{invalid}}").is_err());
	}

	#[test]
	fn parse_empty_modules_is_ok() {
		let toml = r#"
			[meta]
			ns = "test"
			db = "main"
			system_db = "_system"

			# No [[modules]] sections
		"#;
		let manifest: Manifest = toml::from_str(toml).unwrap();
		assert!(manifest.modules.is_empty());
	}

	#[test]
	fn parse_module_with_empty_depends_on() {
		let toml = r#"
			[meta]
			ns = "test"
			db = "main"
			system_db = "_system"

			[[modules]]
			name = "core"
			path = "schema/core"
			depends_on = []
		"#;
		let manifest: Manifest = toml::from_str(toml).unwrap();
		assert!(manifest.modules[0].depends_on.is_empty());
	}

	#[test]
	fn parse_module_without_depends_on() {
		let toml = r#"
			[meta]
			ns = "test"
			db = "main"
			system_db = "_system"

			[[modules]]
			name = "core"
			path = "schema/core"
		"#;
		let manifest: Manifest = toml::from_str(toml).unwrap();
		assert!(manifest.modules[0].depends_on.is_empty());
	}

	#[test]
	fn parse_module_with_multiple_deps() {
		let toml = r#"
			[meta]
			ns = "test"
			db = "main"
			system_db = "_system"

			[[modules]]
			name = "analytics"
			path = "schema/analytics"
			depends_on = ["_shared", "entity", "events"]
		"#;
		let manifest: Manifest = toml::from_str(toml).unwrap();
		assert_eq!(manifest.modules[0].depends_on.len(), 3);
	}

	#[test]
	fn migrations_dir_path() {
		let manifest = Manifest {
			meta: ManifestMeta {
				ns: "test".into(),
				db: "main".into(),
				system_db: "_system".into(),
				surrealdb: None,
			},
			modules: vec![],
			root: Some(std::path::PathBuf::from("/project/surql")),
			preloaded_migrations: None,
			preloaded_modules: None,
		};
		assert_eq!(
			manifest.migrations_dir().unwrap(),
			std::path::PathBuf::from("/project/surql/migrations")
		);
		assert_eq!(
			manifest.generated_dir().unwrap(),
			std::path::PathBuf::from("/project/surql/generated")
		);
	}

	#[test]
	fn embedded_manifest_has_no_root() {
		let manifest = Manifest::builder()
			.meta("test", "main", "_system")
			.build()
			.unwrap();
		assert!(manifest.root.is_none());
		assert!(manifest.root_path().is_err());
		assert!(manifest.migrations_dir().is_err());
	}

	// ─── Builder tests ───

	#[test]
	fn builder_minimal() {
		let manifest = Manifest::builder()
			.meta("ns", "db", "_sys")
			.build()
			.unwrap();
		assert_eq!(manifest.meta.ns, "ns");
		assert_eq!(manifest.meta.db, "db");
		assert_eq!(manifest.meta.system_db, "_sys");
		assert!(manifest.preloaded_migrations.as_ref().unwrap().is_empty());
		assert!(manifest.preloaded_modules.as_ref().unwrap().is_empty());
	}

	#[test]
	fn builder_requires_meta() {
		let result = ManifestBuilder::default().build();
		assert!(result.is_err());
	}

	#[test]
	fn builder_with_migrations() {
		let manifest = Manifest::builder()
			.meta("ns", "db", "_sys")
			.migration(1, "seed", "CREATE user SET name = 'Alice';")
			.migration(2, "more", "CREATE user SET name = 'Bob';")
			.build()
			.unwrap();
		let migrations = manifest.preloaded_migrations.as_ref().unwrap();
		assert_eq!(migrations.len(), 2);
		assert_eq!(migrations[0].version, 1);
		assert_eq!(migrations[1].version, 2);
		assert!(!migrations[0].checksum.is_empty());
	}

	#[test]
	fn builder_sorts_migrations() {
		let manifest = Manifest::builder()
			.meta("ns", "db", "_sys")
			.migration(2, "second", "SELECT 2;")
			.migration(1, "first", "SELECT 1;")
			.build()
			.unwrap();
		let migrations = manifest.preloaded_migrations.as_ref().unwrap();
		assert_eq!(migrations[0].version, 1);
		assert_eq!(migrations[1].version, 2);
	}

	#[test]
	fn builder_warns_on_gap_but_succeeds() {
		let result = Manifest::builder()
			.meta("ns", "db", "_sys")
			.migration(1, "a", "SELECT 1;")
			.migration(3, "c", "SELECT 3;") // gap — warns but succeeds
			.build();
		assert!(result.is_ok());
		let manifest = result.unwrap();
		let migrations = manifest.preloaded_migrations.as_ref().unwrap();
		assert_eq!(migrations.len(), 2);
		assert_eq!(migrations[0].version, 1);
		assert_eq!(migrations[1].version, 3);
	}

	#[test]
	fn builder_with_modules() {
		let manifest = Manifest::builder()
			.meta("ns", "db", "_sys")
			.module("_shared", &[], &["DEFINE ANALYZER a;"])
			.module(
				"entity",
				&["_shared"],
				&["DEFINE TABLE user;", "DEFINE FIELD name ON user;"],
			)
			.build()
			.unwrap();
		let modules = manifest.preloaded_modules.as_ref().unwrap();
		assert_eq!(modules.len(), 2);
		// Topological order: _shared first
		assert_eq!(modules[0].name, "_shared");
		assert_eq!(modules[1].name, "entity");
		assert!(modules[1].content.contains("DEFINE TABLE user;"));
		assert!(modules[1].content.contains("DEFINE FIELD name ON user;"));
	}

	#[test]
	fn builder_sorts_modules_by_deps() {
		let manifest = Manifest::builder()
			.meta("ns", "db", "_sys")
			.module("c", &["b"], &["SELECT 3;"])
			.module("a", &[], &["SELECT 1;"])
			.module("b", &["a"], &["SELECT 2;"])
			.build()
			.unwrap();
		let modules = manifest.preloaded_modules.as_ref().unwrap();
		assert_eq!(modules[0].name, "a");
		assert_eq!(modules[1].name, "b");
		assert_eq!(modules[2].name, "c");
	}

	#[test]
	fn builder_rejects_cyclic_deps() {
		let result = Manifest::builder()
			.meta("ns", "db", "_sys")
			.module("a", &["b"], &["SELECT 1;"])
			.module("b", &["a"], &["SELECT 2;"])
			.build();
		assert!(result.is_err());
	}
}
