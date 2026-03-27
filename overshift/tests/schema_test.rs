use overshift::Manifest;

fn fixture_path() -> std::path::PathBuf {
	std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_project")
}
use overshift::schema::{extract_function_names, load_schema_modules};

#[test]
fn load_sample_schema_modules() {
	let manifest = Manifest::load(fixture_path()).unwrap();
	let modules = load_schema_modules(&manifest).unwrap();

	// Loaded in dependency order: _shared first, then entity
	assert_eq!(modules.len(), 2);
	assert_eq!(modules[0].name, "_shared");
	assert_eq!(modules[1].name, "entity");

	// _shared has analyzer content
	assert!(modules[0].content.contains("ANALYZER"));

	// entity has tables, indexes, functions
	assert!(modules[1].content.contains("TABLE"));
	assert!(modules[1].content.contains("INDEX"));
	assert!(modules[1].content.contains("FUNCTION"));
}

#[test]
fn extract_functions_from_sample() {
	let manifest = Manifest::load(fixture_path()).unwrap();
	let modules = load_schema_modules(&manifest).unwrap();
	let functions = extract_function_names(&modules).unwrap();

	assert_eq!(functions.len(), 2);
	assert!(functions.contains(&"greet".to_string()));
	assert!(functions.contains(&"user_count".to_string()));
}

#[test]
fn extract_functions_empty_modules() {
	let functions = extract_function_names(&[]).unwrap();
	assert!(functions.is_empty());
}

#[test]
fn load_schema_module_tracks_files() {
	let manifest = Manifest::load(fixture_path()).unwrap();
	let modules = load_schema_modules(&manifest).unwrap();

	// _shared has 1 file
	assert_eq!(modules[0].files.len(), 1);
	assert!(modules[0].files[0].contains("analyzers.surql"));

	// entity has 3 files
	assert_eq!(modules[1].files.len(), 3);
}

#[test]
fn load_schema_rejects_missing_module_dir() {
	let tmp = tempfile::tempdir().unwrap();
	std::fs::write(
		tmp.path().join("manifest.toml"),
		r#"
		[meta]
		ns = "test"
		db = "main"
		system_db = "_system"

		[[modules]]
		name = "ghost"
		path = "schema/ghost"
	"#,
	)
	.unwrap();

	let manifest = Manifest::load(tmp.path()).unwrap();
	let result = load_schema_modules(&manifest);
	assert!(result.is_err());
	assert!(result.unwrap_err().to_string().contains("does not exist"));
}

#[test]
fn load_schema_rejects_empty_module() {
	let tmp = tempfile::tempdir().unwrap();
	std::fs::create_dir_all(tmp.path().join("schema/empty")).unwrap();
	std::fs::write(
		tmp.path().join("manifest.toml"),
		r#"
		[meta]
		ns = "test"
		db = "main"
		system_db = "_system"

		[[modules]]
		name = "empty"
		path = "schema/empty"
	"#,
	)
	.unwrap();

	let manifest = Manifest::load(tmp.path()).unwrap();
	let result = load_schema_modules(&manifest);
	assert!(result.is_err());
	assert!(result.unwrap_err().to_string().contains("no .surql files"));
}

#[test]
fn load_schema_ignores_non_surql_files() {
	let tmp = tempfile::tempdir().unwrap();
	let mod_dir = tmp.path().join("schema/core");
	std::fs::create_dir_all(&mod_dir).unwrap();
	std::fs::write(mod_dir.join("table.surql"), "DEFINE TABLE t SCHEMAFULL;").unwrap();
	std::fs::write(mod_dir.join("README.md"), "# docs").unwrap();
	std::fs::write(mod_dir.join("notes.txt"), "some notes").unwrap();

	std::fs::write(
		tmp.path().join("manifest.toml"),
		r#"
		[meta]
		ns = "test"
		db = "main"
		system_db = "_system"

		[[modules]]
		name = "core"
		path = "schema/core"
	"#,
	)
	.unwrap();

	let manifest = Manifest::load(tmp.path()).unwrap();
	let modules = load_schema_modules(&manifest).unwrap();
	assert_eq!(modules.len(), 1);
	// Only table.surql content
	assert!(modules[0].content.contains("DEFINE TABLE"));
	assert!(!modules[0].content.contains("# docs"));
}

#[test]
fn load_schema_no_modules_is_ok() {
	let tmp = tempfile::tempdir().unwrap();
	std::fs::write(
		tmp.path().join("manifest.toml"),
		r#"
		[meta]
		ns = "test"
		db = "main"
		system_db = "_system"
	"#,
	)
	.unwrap();

	let manifest = Manifest::load(tmp.path()).unwrap();
	let modules = load_schema_modules(&manifest).unwrap();
	assert!(modules.is_empty());
}

#[test]
fn load_schema_concatenates_files_in_order() {
	let tmp = tempfile::tempdir().unwrap();
	let mod_dir = tmp.path().join("schema/ordered");
	std::fs::create_dir_all(&mod_dir).unwrap();
	std::fs::write(mod_dir.join("01_first.surql"), "-- FIRST").unwrap();
	std::fs::write(mod_dir.join("02_second.surql"), "-- SECOND").unwrap();
	std::fs::write(mod_dir.join("03_third.surql"), "-- THIRD").unwrap();

	std::fs::write(
		tmp.path().join("manifest.toml"),
		r#"
		[meta]
		ns = "test"
		db = "main"
		system_db = "_system"

		[[modules]]
		name = "ordered"
		path = "schema/ordered"
	"#,
	)
	.unwrap();

	let manifest = Manifest::load(tmp.path()).unwrap();
	let modules = load_schema_modules(&manifest).unwrap();

	// Files sorted by name, concatenated in order
	let content = &modules[0].content;
	let first_pos = content.find("FIRST").unwrap();
	let second_pos = content.find("SECOND").unwrap();
	let third_pos = content.find("THIRD").unwrap();
	assert!(first_pos < second_pos);
	assert!(second_pos < third_pos);
}

#[test]
fn load_schema_with_nested_surql_files() {
	let tmp = tempfile::tempdir().unwrap();
	let mod_dir = tmp.path().join("schema/deep");
	std::fs::create_dir_all(mod_dir.join("subdir")).unwrap();
	std::fs::write(mod_dir.join("root.surql"), "-- ROOT").unwrap();
	std::fs::write(mod_dir.join("subdir/nested.surql"), "-- NESTED").unwrap();

	std::fs::write(
		tmp.path().join("manifest.toml"),
		r#"
		[meta]
		ns = "test"
		db = "main"
		system_db = "_system"

		[[modules]]
		name = "deep"
		path = "schema/deep"
	"#,
	)
	.unwrap();

	let manifest = Manifest::load(tmp.path()).unwrap();
	let modules = load_schema_modules(&manifest).unwrap();

	// WalkDir recurses into subdirs
	assert!(modules[0].content.contains("ROOT"));
	assert!(modules[0].content.contains("NESTED"));
	assert_eq!(modules[0].files.len(), 2);
}
