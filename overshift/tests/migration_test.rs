use overshift::Manifest;
use overshift::migration::{compute_checksum, discover_migrations};

fn fixture_path() -> std::path::PathBuf {
	std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_project")
}

// ─── Discovery from fixtures ───

#[test]
fn discover_sample_migrations() {
	let migrations = discover_migrations(&fixture_path()).unwrap();
	assert_eq!(migrations.len(), 2);
	assert_eq!(migrations[0].version, 1);
	assert_eq!(migrations[0].name, "seed");
	assert_eq!(migrations[1].version, 2);
	assert_eq!(migrations[1].name, "more_data");

	// Checksums are deterministic
	assert_eq!(migrations[0].checksum.len(), 64);
	assert_ne!(migrations[0].checksum, migrations[1].checksum);
}

#[test]
fn discover_empty_dir() {
	let tmp = tempfile::tempdir().unwrap();
	let migrations = discover_migrations(tmp.path()).unwrap();
	assert!(migrations.is_empty());
}

#[test]
fn discover_nonexistent_dir() {
	let migrations = discover_migrations(std::path::Path::new("/nonexistent")).unwrap();
	assert!(migrations.is_empty());
}

// ─── Discovery with custom fixtures ───

#[test]
fn discover_single_migration() {
	let tmp = tempfile::tempdir().unwrap();
	let mig_dir = tmp.path().join("migrations");
	std::fs::create_dir(&mig_dir).unwrap();
	std::fs::write(
		mig_dir.join("v001_init.surql"),
		"CREATE user SET name = 'test';",
	)
	.unwrap();

	let migrations = discover_migrations(tmp.path()).unwrap();
	assert_eq!(migrations.len(), 1);
	assert_eq!(migrations[0].version, 1);
	assert_eq!(migrations[0].name, "init");
	assert_eq!(
		migrations[0].checksum,
		compute_checksum("CREATE user SET name = 'test';")
	);
}

#[test]
fn discover_many_migrations_sorted() {
	let tmp = tempfile::tempdir().unwrap();
	let mig_dir = tmp.path().join("migrations");
	std::fs::create_dir(&mig_dir).unwrap();

	// Write out of order
	for i in [5, 3, 1, 4, 2] {
		std::fs::write(
			mig_dir.join(format!("v{i:03}_step_{i}.surql")),
			format!("-- migration {i}"),
		)
		.unwrap();
	}

	let migrations = discover_migrations(tmp.path()).unwrap();
	assert_eq!(migrations.len(), 5);
	for (i, m) in migrations.iter().enumerate() {
		assert_eq!(m.version, (i + 1) as u32);
	}
}

#[test]
fn discover_ignores_non_surql_files() {
	let tmp = tempfile::tempdir().unwrap();
	let mig_dir = tmp.path().join("migrations");
	std::fs::create_dir(&mig_dir).unwrap();

	std::fs::write(mig_dir.join("v001_init.surql"), "SELECT 1;").unwrap();
	std::fs::write(mig_dir.join("README.md"), "# docs").unwrap();
	std::fs::write(mig_dir.join("notes.txt"), "notes").unwrap();
	std::fs::write(mig_dir.join(".gitkeep"), "").unwrap();

	let migrations = discover_migrations(tmp.path()).unwrap();
	assert_eq!(migrations.len(), 1);
}

#[test]
fn discover_ignores_nested_dirs() {
	let tmp = tempfile::tempdir().unwrap();
	let mig_dir = tmp.path().join("migrations");
	std::fs::create_dir_all(mig_dir.join("subfolder")).unwrap();

	std::fs::write(mig_dir.join("v001_init.surql"), "SELECT 1;").unwrap();
	std::fs::write(mig_dir.join("subfolder/v002_nested.surql"), "SELECT 2;").unwrap();

	let migrations = discover_migrations(tmp.path()).unwrap();
	// Only depth-1 files
	assert_eq!(migrations.len(), 1);
	assert_eq!(migrations[0].version, 1);
}

#[test]
fn discover_rejects_bad_filename() {
	let tmp = tempfile::tempdir().unwrap();
	let mig_dir = tmp.path().join("migrations");
	std::fs::create_dir(&mig_dir).unwrap();

	// Bad filename: no 'v' prefix
	std::fs::write(mig_dir.join("001_init.surql"), "SELECT 1;").unwrap();

	let result = discover_migrations(tmp.path());
	assert!(result.is_err());
}

#[test]
fn discover_warns_on_gap_but_succeeds() {
	let tmp = tempfile::tempdir().unwrap();
	let mig_dir = tmp.path().join("migrations");
	std::fs::create_dir(&mig_dir).unwrap();

	std::fs::write(mig_dir.join("v001_a.surql"), "SELECT 1;").unwrap();
	std::fs::write(mig_dir.join("v003_c.surql"), "SELECT 3;").unwrap();

	let result = discover_migrations(tmp.path());
	assert!(result.is_ok());
	let migrations = result.unwrap();
	assert_eq!(migrations.len(), 2);
	assert_eq!(migrations[0].version, 1);
	assert_eq!(migrations[1].version, 3);
}

#[test]
fn discover_rejects_duplicate_versions() {
	let tmp = tempfile::tempdir().unwrap();
	let mig_dir = tmp.path().join("migrations");
	std::fs::create_dir(&mig_dir).unwrap();

	std::fs::write(mig_dir.join("v001_first.surql"), "SELECT 1;").unwrap();
	std::fs::write(mig_dir.join("v001_second.surql"), "SELECT 2;").unwrap();

	let result = discover_migrations(tmp.path());
	assert!(result.is_err());
}

#[test]
fn discover_checksum_changes_with_content() {
	let tmp = tempfile::tempdir().unwrap();
	let mig_dir = tmp.path().join("migrations");
	std::fs::create_dir(&mig_dir).unwrap();

	std::fs::write(mig_dir.join("v001_init.surql"), "version A").unwrap();
	let m1 = discover_migrations(tmp.path()).unwrap();

	std::fs::write(mig_dir.join("v001_init.surql"), "version B").unwrap();
	let m2 = discover_migrations(tmp.path()).unwrap();

	assert_ne!(m1[0].checksum, m2[0].checksum);
}

#[test]
fn discover_preserves_content() {
	let tmp = tempfile::tempdir().unwrap();
	let mig_dir = tmp.path().join("migrations");
	std::fs::create_dir(&mig_dir).unwrap();

	let sql = "DEFINE TABLE user SCHEMAFULL;\nDEFINE FIELD name ON user TYPE string;\n";
	std::fs::write(mig_dir.join("v001_schema.surql"), sql).unwrap();

	let migrations = discover_migrations(tmp.path()).unwrap();
	assert_eq!(migrations[0].content, sql);
}

#[test]
fn discover_handles_large_version_number() {
	let tmp = tempfile::tempdir().unwrap();
	let mig_dir = tmp.path().join("migrations");
	std::fs::create_dir(&mig_dir).unwrap();

	// Create v001 through v100
	for i in 1..=100 {
		std::fs::write(
			mig_dir.join(format!("v{i:03}_step{i}.surql")),
			format!("SELECT {i};"),
		)
		.unwrap();
	}

	let migrations = discover_migrations(tmp.path()).unwrap();
	assert_eq!(migrations.len(), 100);
	assert_eq!(migrations[99].version, 100);
}

// ─── Down migration discovery ───

#[test]
fn discover_loads_down_content_when_present() {
	let tmp = tempfile::tempdir().unwrap();
	let mig_dir = tmp.path().join("migrations");
	std::fs::create_dir(&mig_dir).unwrap();

	std::fs::write(
		mig_dir.join("v001_create_user.surql"),
		"DEFINE TABLE user SCHEMAFULL;",
	)
	.unwrap();
	std::fs::write(
		mig_dir.join("v001_create_user.down.surql"),
		"REMOVE TABLE user;",
	)
	.unwrap();

	let migrations = discover_migrations(tmp.path()).unwrap();
	assert_eq!(migrations.len(), 1);
	assert_eq!(
		migrations[0].down_content.as_deref(),
		Some("REMOVE TABLE user;")
	);
}

#[test]
fn discover_sets_down_content_none_when_absent() {
	let tmp = tempfile::tempdir().unwrap();
	let mig_dir = tmp.path().join("migrations");
	std::fs::create_dir(&mig_dir).unwrap();

	std::fs::write(mig_dir.join("v001_init.surql"), "SELECT 1;").unwrap();

	let migrations = discover_migrations(tmp.path()).unwrap();
	assert_eq!(migrations.len(), 1);
	assert!(migrations[0].down_content.is_none());
}

#[test]
fn discover_does_not_count_down_file_as_separate_migration() {
	let tmp = tempfile::tempdir().unwrap();
	let mig_dir = tmp.path().join("migrations");
	std::fs::create_dir(&mig_dir).unwrap();

	std::fs::write(mig_dir.join("v001_init.surql"), "SELECT 1;").unwrap();
	std::fs::write(mig_dir.join("v001_init.down.surql"), "SELECT 0;").unwrap();
	std::fs::write(mig_dir.join("v002_more.surql"), "SELECT 2;").unwrap();
	std::fs::write(mig_dir.join("v002_more.down.surql"), "SELECT 0;").unwrap();

	let migrations = discover_migrations(tmp.path()).unwrap();
	assert_eq!(
		migrations.len(),
		2,
		"down.surql files should not be counted as migrations"
	);
	assert_eq!(migrations[0].version, 1);
	assert_eq!(migrations[1].version, 2);
}

// ─── Conflict detection ───

#[test]
fn should_detect_duplicate_versions() {
	let tmp = tempfile::tempdir().unwrap();
	let mig_dir = tmp.path().join("migrations");
	std::fs::create_dir(&mig_dir).unwrap();

	std::fs::write(mig_dir.join("v003_alice.surql"), "SELECT 'alice';").unwrap();
	std::fs::write(mig_dir.join("v003_bob.surql"), "SELECT 'bob';").unwrap();
	std::fs::write(mig_dir.join("v001_init.surql"), "SELECT 1;").unwrap();
	std::fs::write(mig_dir.join("v002_seed.surql"), "SELECT 2;").unwrap();

	let result = discover_migrations(tmp.path());
	assert!(result.is_err());

	let err_msg = result.unwrap_err().to_string();
	assert!(
		err_msg.contains("v003_alice.surql"),
		"error should mention v003_alice.surql, got: {err_msg}"
	);
	assert!(
		err_msg.contains("v003_bob.surql"),
		"error should mention v003_bob.surql, got: {err_msg}"
	);
	assert!(
		err_msg.contains("duplicate"),
		"error should mention 'duplicate', got: {err_msg}"
	);
}

#[test]
fn should_warn_on_version_gap() {
	let tmp = tempfile::tempdir().unwrap();
	let mig_dir = tmp.path().join("migrations");
	std::fs::create_dir(&mig_dir).unwrap();

	std::fs::write(mig_dir.join("v001_init.surql"), "SELECT 1;").unwrap();
	std::fs::write(mig_dir.join("v003_skip.surql"), "SELECT 3;").unwrap();

	let result = discover_migrations(tmp.path());
	assert!(
		result.is_ok(),
		"gaps should warn but not fail: {:?}",
		result.err()
	);

	let migrations = result.unwrap();
	assert_eq!(migrations.len(), 2);
	assert_eq!(migrations[0].version, 1);
	assert_eq!(migrations[1].version, 3);
}

#[tokio::test]
async fn should_reject_duplicate_in_plan() {
	let db = surrealdb::engine::any::connect("mem://").await.unwrap();
	db.use_ns("test_dup_plan").use_db("main").await.unwrap();

	let tmp = tempfile::tempdir().unwrap();
	let mig_dir = tmp.path().join("migrations");
	std::fs::create_dir(&mig_dir).unwrap();

	std::fs::write(mig_dir.join("v001_init.surql"), "-- init").unwrap();
	std::fs::write(mig_dir.join("v002_seed.surql"), "-- seed").unwrap();
	std::fs::write(mig_dir.join("v003_data.surql"), "-- original data").unwrap();
	std::fs::write(
		tmp.path().join("manifest.toml"),
		r#"
		[meta]
		ns = "test_dup_plan"
		db = "main"
		system_db = "_system"
	"#,
	)
	.unwrap();

	let manifest = Manifest::load(tmp.path()).unwrap();

	// Apply all three migrations
	let plan = overshift::plan(&db, &manifest).await.unwrap();
	assert_eq!(plan.pending_migrations.len(), 3);
	plan.apply(&db).await.unwrap();

	// Tamper with v003 content on disk after it was applied
	std::fs::write(mig_dir.join("v003_data.surql"), "-- MODIFIED data").unwrap();

	// Plan should detect that applied v003 checksum doesn't match filesystem v003
	let result = overshift::plan(&db, &manifest).await;
	assert!(
		result.is_err(),
		"plan should reject when applied migration checksum differs from filesystem"
	);
	let err = result.unwrap_err().to_string();
	assert!(
		err.contains("checksum mismatch"),
		"expected checksum mismatch, got: {err}"
	);
}
