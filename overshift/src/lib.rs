//! `overshift` — shared migration engine for the overrealdb ecosystem.
//!
//! Manages both **declarative schema** (re-applied with `DEFINE ... OVERWRITE`)
//! and **imperative migrations** (versioned, checksummed, one-shot) for SurrealDB.
//!
//! # Usage
//!
//! ```rust,ignore
//! use surrealdb::engine::any;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let db = any::connect("ws://localhost:8000").await?;
//!
//!     let manifest = overshift::Manifest::load("surql/")?;
//!
//!     // Preview what will happen (dry-run)
//!     let plan = overshift::plan(&db, &manifest).await?;
//!     plan.print();
//!
//!     // Apply migrations + schema
//!     let result = plan.apply(&db).await?;
//!     println!("Applied {} migrations, {} modules", result.applied_migrations, result.applied_modules);
//!
//!     Ok(())
//! }
//! ```

pub mod changelog;
pub mod error;
pub mod lock;
pub mod manifest;
pub mod migration;
pub mod plan;
pub mod schema;
#[cfg(feature = "shadow")]
pub mod shadow;
pub mod snapshot;
pub mod validate;

pub use error::{Error, Result};
#[allow(deprecated)]
pub use lock::MigrationLock;
pub use lock::SurrealLock;
pub use manifest::{Manifest, ManifestBuilder};
pub use migration::{AppliedMigration, Migration, compute_checksum};
pub use plan::{ApplyResult, Plan, RollbackResult, plan, rollback};
pub use schema::SchemaModule;
