//! Rollback tests — require either Docker or in-memory SurrealDB.

#![cfg(any(feature = "validate-docker", feature = "validate-mem"))]

mod common;

use overshift::Manifest;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

fn connect() -> Surreal<Any> {
	#[cfg(feature = "validate-docker")]
	{
		return common::docker::connect();
	}
	#[cfg(all(feature = "validate-mem", not(feature = "validate-docker")))]
	{
		return common::mem::connect();
	}
}

fn runtime() -> &'static tokio::runtime::Runtime {
	#[cfg(feature = "validate-docker")]
	{
		return common::docker::runtime();
	}
	#[cfg(all(feature = "validate-mem", not(feature = "validate-docker")))]
	{
		return common::mem::runtime();
	}
}

#[test]
fn should_rollback_single_migration() {
	let rt = runtime();
	let db = connect();

	let manifest = Manifest::builder()
		.meta("test_rollback_single", "main", "_system")
		.migration_with_down(
			1,
			"create_user",
			"DEFINE TABLE IF NOT EXISTS user SCHEMAFULL;",
			"REMOVE TABLE IF EXISTS user;",
		)
		.migration_with_down(
			2,
			"create_post",
			"DEFINE TABLE IF NOT EXISTS post SCHEMAFULL;",
			"REMOVE TABLE IF EXISTS post;",
		)
		.build()
		.unwrap();

	rt.block_on(async {
		// Apply both migrations
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		let result = plan.apply(&db).await.unwrap();
		assert_eq!(result.applied_migrations, 2);

		// Rollback to v001 — should remove v002
		let rb = overshift::rollback(&db, &manifest, 1).await.unwrap();
		assert_eq!(rb.rolled_back.len(), 1);
		assert!(rb.rolled_back[0].contains("create_post"));
		assert_eq!(rb.target_version, 1);
		assert!(rb.errors.is_empty());

		// Verify migration_lock only has v001
		db.use_ns("test_rollback_single")
			.use_db("_system")
			.await
			.unwrap();
		let mut response = db
			.query("SELECT version FROM migration_lock ORDER BY version")
			.await
			.unwrap();
		let rows: Vec<serde_json::Value> = response.take(0).unwrap();
		assert_eq!(rows.len(), 1);
		assert_eq!(rows[0].get("version").and_then(|v| v.as_i64()), Some(1));
	});
}

#[test]
fn should_rollback_with_down_file() {
	let rt = runtime();
	let db = connect();

	let manifest = Manifest::builder()
		.meta("test_rollback_down", "main", "_system")
		.migration_with_down(
			1,
			"create_item",
			"DEFINE TABLE IF NOT EXISTS item SCHEMAFULL;\
			 DEFINE FIELD IF NOT EXISTS name ON item TYPE string;",
			"REMOVE TABLE IF EXISTS item;",
		)
		.build()
		.unwrap();

	rt.block_on(async {
		// Apply
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		plan.apply(&db).await.unwrap();

		// Verify table exists
		db.use_ns("test_rollback_down")
			.use_db("main")
			.await
			.unwrap();
		let mut response = db.query("INFO FOR DB").await.unwrap();
		let info: Option<serde_json::Value> = response.take(0).unwrap();
		let tables = info
			.as_ref()
			.and_then(|v| v.get("tables"))
			.and_then(|t| t.as_object())
			.unwrap();
		assert!(tables.contains_key("item"));

		// Rollback to v000 (before any migration)
		let rb = overshift::rollback(&db, &manifest, 0).await.unwrap();
		assert_eq!(rb.rolled_back.len(), 1);
		assert!(rb.errors.is_empty());

		// Verify table was removed by down migration
		db.use_ns("test_rollback_down")
			.use_db("main")
			.await
			.unwrap();
		let mut response = db.query("INFO FOR DB").await.unwrap();
		let info: Option<serde_json::Value> = response.take(0).unwrap();
		let tables = info
			.as_ref()
			.and_then(|v| v.get("tables"))
			.and_then(|t| t.as_object())
			.unwrap();
		assert!(
			!tables.contains_key("item"),
			"table 'item' should have been removed by rollback"
		);

		// Verify changelog records the rollback
		db.use_ns("test_rollback_down")
			.use_db("_system")
			.await
			.unwrap();
		let mut response = db
			.query("SELECT * FROM changelog WHERE type = 'rollback'")
			.await
			.unwrap();
		let rows: Vec<serde_json::Value> = response.take(0).unwrap();
		assert_eq!(rows.len(), 1);
	});
}

#[test]
fn should_error_on_rollback_without_down_file() {
	let rt = runtime();
	let db = connect();

	// Migration without down content
	let manifest = Manifest::builder()
		.meta("test_rollback_no_down", "main", "_system")
		.migration(
			1,
			"create_widget",
			"DEFINE TABLE IF NOT EXISTS widget SCHEMAFULL;",
		)
		.build()
		.unwrap();

	rt.block_on(async {
		// Apply
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		plan.apply(&db).await.unwrap();

		// Rollback — should report error for missing .down.surql
		let rb = overshift::rollback(&db, &manifest, 0).await.unwrap();
		assert!(rb.rolled_back.is_empty());
		assert_eq!(rb.errors.len(), 1);
		assert!(
			rb.errors[0].contains("no .down.surql"),
			"expected 'no .down.surql' in error, got: {}",
			rb.errors[0]
		);

		// Verify migration_lock is unchanged (migration not removed)
		db.use_ns("test_rollback_no_down")
			.use_db("_system")
			.await
			.unwrap();
		let mut response = db
			.query("SELECT version FROM migration_lock")
			.await
			.unwrap();
		let rows: Vec<serde_json::Value> = response.take(0).unwrap();
		assert_eq!(
			rows.len(),
			1,
			"migration should not have been removed from migration_lock"
		);
	});
}

#[test]
fn should_rollback_multiple_in_reverse_order() {
	let rt = runtime();
	let db = connect();

	let manifest = Manifest::builder()
		.meta("test_rollback_multi", "main", "_system")
		.migration_with_down(
			1,
			"table_a",
			"DEFINE TABLE IF NOT EXISTS a SCHEMAFULL;",
			"REMOVE TABLE IF EXISTS a;",
		)
		.migration_with_down(
			2,
			"table_b",
			"DEFINE TABLE IF NOT EXISTS b SCHEMAFULL;",
			"REMOVE TABLE IF EXISTS b;",
		)
		.migration_with_down(
			3,
			"table_c",
			"DEFINE TABLE IF NOT EXISTS c SCHEMAFULL;",
			"REMOVE TABLE IF EXISTS c;",
		)
		.build()
		.unwrap();

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		plan.apply(&db).await.unwrap();

		// Rollback to v001 — should remove v003 then v002 (reverse order)
		let rb = overshift::rollback(&db, &manifest, 1).await.unwrap();
		assert_eq!(rb.rolled_back.len(), 2);
		// v003 should be rolled back first (reverse order)
		assert!(rb.rolled_back[0].contains("table_c"));
		assert!(rb.rolled_back[1].contains("table_b"));
		assert!(rb.errors.is_empty());

		// Verify only v001 remains in migration_lock
		db.use_ns("test_rollback_multi")
			.use_db("_system")
			.await
			.unwrap();
		let mut response = db
			.query("SELECT version FROM migration_lock ORDER BY version")
			.await
			.unwrap();
		let rows: Vec<serde_json::Value> = response.take(0).unwrap();
		assert_eq!(rows.len(), 1);
		assert_eq!(rows[0].get("version").and_then(|v| v.as_i64()), Some(1));
	});
}

#[test]
fn should_noop_when_already_at_target_version() {
	let rt = runtime();
	let db = connect();

	let manifest = Manifest::builder()
		.meta("test_rollback_noop", "main", "_system")
		.migration_with_down(
			1,
			"init",
			"DEFINE TABLE IF NOT EXISTS noop_tbl SCHEMAFULL;",
			"REMOVE TABLE IF EXISTS noop_tbl;",
		)
		.build()
		.unwrap();

	rt.block_on(async {
		let plan = overshift::plan(&db, &manifest).await.unwrap();
		plan.apply(&db).await.unwrap();

		// Rollback to v001 when we're already at v001 — nothing to do
		let rb = overshift::rollback(&db, &manifest, 1).await.unwrap();
		assert!(rb.rolled_back.is_empty());
		assert!(rb.errors.is_empty());
	});
}
