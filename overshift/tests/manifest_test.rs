use overshift::Manifest;

fn fixture_path() -> std::path::PathBuf {
	std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_project")
}

#[test]
fn load_sample_manifest() {
	let manifest = Manifest::load(fixture_path()).unwrap();
	assert_eq!(manifest.meta.ns, "test_app");
	assert_eq!(manifest.meta.db, "main");
	assert_eq!(manifest.meta.system_db, "_system");
	assert_eq!(manifest.modules.len(), 2);
	assert_eq!(manifest.modules[0].name, "_shared");
	assert_eq!(manifest.modules[1].name, "entity");
	assert_eq!(manifest.modules[1].depends_on, vec!["_shared"]);
}

#[test]
fn load_nonexistent_manifest() {
	let result = Manifest::load("/nonexistent/path");
	assert!(result.is_err());
}

#[test]
fn load_sets_root_path() {
	let manifest = Manifest::load(fixture_path()).unwrap();
	assert_eq!(manifest.root, Some(fixture_path()));
}

#[test]
fn load_manifest_from_tempdir() {
	let tmp = tempfile::tempdir().unwrap();
	std::fs::write(
		tmp.path().join("manifest.toml"),
		r#"
		[meta]
		ns = "myns"
		db = "mydb"
		system_db = "_sys"
	"#,
	)
	.unwrap();

	let manifest = Manifest::load(tmp.path()).unwrap();
	assert_eq!(manifest.meta.ns, "myns");
	assert_eq!(manifest.meta.db, "mydb");
	assert_eq!(manifest.meta.system_db, "_sys");
	assert!(manifest.modules.is_empty());
}

#[test]
fn load_rejects_empty_file() {
	let tmp = tempfile::tempdir().unwrap();
	std::fs::write(tmp.path().join("manifest.toml"), "").unwrap();

	let result = Manifest::load(tmp.path());
	assert!(result.is_err());
}

#[test]
fn load_rejects_invalid_toml() {
	let tmp = tempfile::tempdir().unwrap();
	std::fs::write(tmp.path().join("manifest.toml"), "{{{{").unwrap();

	let result = Manifest::load(tmp.path());
	assert!(result.is_err());
}

#[test]
fn load_rejects_missing_meta_section() {
	let tmp = tempfile::tempdir().unwrap();
	std::fs::write(
		tmp.path().join("manifest.toml"),
		r#"
		[[modules]]
		name = "x"
		path = "schema/x"
	"#,
	)
	.unwrap();

	let result = Manifest::load(tmp.path());
	assert!(result.is_err());
}

#[test]
fn migrations_dir_derived_from_root() {
	let manifest = Manifest::load(fixture_path()).unwrap();
	assert!(manifest.migrations_dir().unwrap().ends_with("migrations"));
}

#[test]
fn generated_dir_derived_from_root() {
	let manifest = Manifest::load(fixture_path()).unwrap();
	assert!(manifest.generated_dir().unwrap().ends_with("generated"));
}
