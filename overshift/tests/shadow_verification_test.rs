//! Shadow DB verification tests — compare real DB state after apply
//! with shadow in-memory DB to detect drift.
//!
//! Requires `validate-docker` or `validate-mem` feature + `shadow` feature.

#![cfg(all(
	any(feature = "validate-docker", feature = "validate-mem"),
	feature = "shadow"
))]

mod common;

use overshift::Manifest;
use overshift::shadow::{apply_to_shadow, verify_against_shadow};
use overshift::validate::{compare_db_info, query_db_info};
use std::sync::atomic::{AtomicU32, Ordering};

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

fn unique_ns(prefix: &str) -> String {
	let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
	format!("{prefix}_{id}")
}

async fn connect() -> surrealdb::Surreal<surrealdb::engine::any::Any> {
	#[cfg(feature = "validate-docker")]
	{
		// Docker connect uses block_on internally; run in spawn_blocking
		tokio::task::spawn_blocking(|| common::docker::connect())
			.await
			.unwrap()
	}
	#[cfg(all(feature = "validate-mem", not(feature = "validate-docker")))]
	{
		tokio::task::spawn_blocking(|| common::mem::connect())
			.await
			.unwrap()
	}
}

fn fixture_path() -> std::path::PathBuf {
	std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_project")
}

fn fixture_manifest(test_name: &str) -> Manifest {
	let mut manifest = Manifest::load(fixture_path()).unwrap();
	manifest.meta.ns = format!("shadow_{test_name}");
	manifest
}

// ─── Core: shadow matches real DB after clean apply ───

#[tokio::test]
async fn shadow_matches_real_db_after_apply() {
	let db = connect().await;
	let manifest = fixture_manifest("match_real");

	let plan = overshift::plan(&db, &manifest).await.unwrap();
	plan.apply(&db).await.unwrap();

	db.use_ns(&manifest.meta.ns)
		.use_db(&manifest.meta.db)
		.await
		.unwrap();
	let real_info = query_db_info(&db).await.unwrap();

	let shadow = apply_to_shadow(&manifest).await.unwrap();
	let diff = compare_db_info(&shadow.db_info, &real_info);

	assert!(
		diff.is_empty(),
		"Shadow should match real DB.\nMissing: {:?}\nExtra: {:?}\nMissing fns: {:?}\nExtra fns: {:?}",
		diff.missing_tables,
		diff.extra_tables,
		diff.missing_functions,
		diff.extra_functions,
	);
}

// ─── Idempotent: second apply still matches ───

#[tokio::test]
async fn shadow_matches_after_idempotent_reapply() {
	let db = connect().await;
	let manifest = fixture_manifest("idempotent");

	let plan1 = overshift::plan(&db, &manifest).await.unwrap();
	plan1.apply(&db).await.unwrap();
	let plan2 = overshift::plan(&db, &manifest).await.unwrap();
	plan2.apply(&db).await.unwrap();

	db.use_ns(&manifest.meta.ns)
		.use_db(&manifest.meta.db)
		.await
		.unwrap();
	let real_info = query_db_info(&db).await.unwrap();
	let shadow = apply_to_shadow(&manifest).await.unwrap();
	let diff = compare_db_info(&shadow.db_info, &real_info);

	assert!(
		diff.is_empty(),
		"Should match after idempotent reapply: {diff}"
	);
}

// ─── Drift detection: manual change on real DB ───

#[tokio::test]
async fn shadow_detects_manual_drift() {
	let db = connect().await;
	let manifest = fixture_manifest("drift");

	let plan = overshift::plan(&db, &manifest).await.unwrap();
	plan.apply(&db).await.unwrap();

	db.use_ns(&manifest.meta.ns)
		.use_db(&manifest.meta.db)
		.await
		.unwrap();
	db.query("DEFINE TABLE rogue_table SCHEMALESS")
		.await
		.unwrap();

	let real_info = query_db_info(&db).await.unwrap();
	let shadow = apply_to_shadow(&manifest).await.unwrap();
	let diff = compare_db_info(&shadow.db_info, &real_info);

	assert!(!diff.is_empty(), "Should detect drift");
	assert!(
		diff.extra_tables.contains(&"rogue_table".to_string()),
		"rogue_table should be extra: {:?}",
		diff.extra_tables
	);
}

// ─── Embedded manifest via builder ───

#[tokio::test]
async fn shadow_verify_with_builder_manifest() {
	let db = connect().await;

	let manifest = Manifest::builder()
		.meta(
			&unique_ns("shadow_builder"),
			&unique_ns("builder_db"),
			"_system",
		)
		.module(
			"core",
			&[],
			&["DEFINE TABLE OVERWRITE widget SCHEMAFULL;\n\
				 DEFINE FIELD OVERWRITE name ON widget TYPE string;"],
		)
		.migration(1, "seed", "CREATE widget:w1 SET name = 'Alpha';")
		.migration(2, "more", "CREATE widget:w2 SET name = 'Beta';")
		.build()
		.unwrap();

	let plan = overshift::plan(&db, &manifest).await.unwrap();
	plan.apply(&db).await.unwrap();

	db.use_ns(&manifest.meta.ns)
		.use_db(&manifest.meta.db)
		.await
		.unwrap();
	let real_info = query_db_info(&db).await.unwrap();
	let diff = verify_against_shadow(&manifest, &real_info).await.unwrap();

	assert!(diff.is_empty(), "Builder manifest should match: {diff}");
}

// ─── Schema-only project (no migrations) ───

#[tokio::test]
async fn shadow_verify_schema_only() {
	let db = connect().await;

	let manifest = Manifest::builder()
		.meta(
			&unique_ns("shadow_schema_only"),
			&unique_ns("schema_db"),
			"_system",
		)
		.module(
			"tables",
			&[],
			&["DEFINE TABLE OVERWRITE config SCHEMAFULL;\n\
				 DEFINE FIELD OVERWRITE key ON config TYPE string;\n\
				 DEFINE FIELD OVERWRITE value ON config TYPE string;\n\
				 DEFINE INDEX OVERWRITE config_key ON config FIELDS key UNIQUE;"],
		)
		.build()
		.unwrap();

	let plan = overshift::plan(&db, &manifest).await.unwrap();
	plan.apply(&db).await.unwrap();

	db.use_ns(&manifest.meta.ns)
		.use_db(&manifest.meta.db)
		.await
		.unwrap();
	let real_info = query_db_info(&db).await.unwrap();
	let diff = verify_against_shadow(&manifest, &real_info).await.unwrap();

	assert!(diff.is_empty(), "Schema-only should match: {diff}");
}

// ─── Multi-module with dependencies ───

#[tokio::test]
async fn shadow_verify_multi_module_deps() {
	let db = connect().await;

	let manifest = Manifest::builder()
		.meta(&unique_ns("shadow_deps"), &unique_ns("deps_db"), "_system")
		.module("shared", &[], &["DEFINE TABLE OVERWRITE audit SCHEMALESS;"])
		.module(
			"core",
			&["shared"],
			&["DEFINE TABLE OVERWRITE task SCHEMAFULL;\n\
				 DEFINE FIELD OVERWRITE title ON task TYPE string;\n\
				 DEFINE FIELD OVERWRITE done ON task TYPE bool DEFAULT false;"],
		)
		.migration(
			1,
			"seed",
			"CREATE task:t1 SET title = 'Setup', done = true;",
		)
		.build()
		.unwrap();

	let plan = overshift::plan(&db, &manifest).await.unwrap();
	plan.apply(&db).await.unwrap();

	db.use_ns(&manifest.meta.ns)
		.use_db(&manifest.meta.db)
		.await
		.unwrap();
	let real_info = query_db_info(&db).await.unwrap();
	let diff = verify_against_shadow(&manifest, &real_info).await.unwrap();

	assert!(diff.is_empty(), "Multi-module should match: {diff}");
}

// ─── Read-only: shadow verification doesn't modify target ───

#[tokio::test]
async fn shadow_verify_is_read_only_on_target() {
	let db = connect().await;

	let manifest = Manifest::builder()
		.meta(
			&unique_ns("shadow_readonly"),
			&unique_ns("readonly_db"),
			"_system",
		)
		.module(
			"core",
			&[],
			&[
				"DEFINE TABLE OVERWRITE item SCHEMAFULL; DEFINE FIELD OVERWRITE name ON item TYPE string;",
			],
		)
		.migration(1, "seed", "CREATE item:a SET name = 'Alpha';")
		.build()
		.unwrap();

	let plan = overshift::plan(&db, &manifest).await.unwrap();
	plan.apply(&db).await.unwrap();

	db.use_ns(&manifest.meta.ns)
		.use_db(&manifest.meta.db)
		.await
		.unwrap();

	let before: Vec<serde_json::Value> = db
		.query("SELECT count() AS c FROM item GROUP ALL")
		.await
		.unwrap()
		.take(0)
		.unwrap();

	// Shadow verify — must NOT write to real DB
	let real_info = query_db_info(&db).await.unwrap();
	let _diff = verify_against_shadow(&manifest, &real_info).await.unwrap();

	let after: Vec<serde_json::Value> = db
		.query("SELECT count() AS c FROM item GROUP ALL")
		.await
		.unwrap()
		.take(0)
		.unwrap();

	assert_eq!(
		before, after,
		"Shadow verification must not modify target DB"
	);
}
