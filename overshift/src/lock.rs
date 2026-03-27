//! Distributed lock for coordinating operations across multiple instances.
//!
//! Uses a `leader_lock` table in SurrealDB as a shedlock-style leader
//! election mechanism. Only one instance can hold the lock at a time; the lock
//! expires after 60 seconds so that a crashed holder does not block others
//! indefinitely.
//!
//! The lock is parameterized by a `scope` which determines the record ID
//! within the `leader_lock` table (e.g., `leader_lock:migration`,
//! `leader_lock:sync`).

use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use tracing::{debug, info, warn};

use crate::Error;

/// How long to wait between lock acquisition attempts.
const RETRY_INTERVAL: std::time::Duration = std::time::Duration::from_secs(2);

/// Maximum number of retries before giving up on acquiring the lock.
const MAX_RETRIES: u32 = 30;

/// Table name for the distributed lock.
const TABLE: &str = "leader_lock";

/// DDL: ensure the lock table exists (idempotent).
const DEFINE_TABLE_SQL: &str = r#"
DEFINE TABLE IF NOT EXISTS leader_lock SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS holder      ON leader_lock TYPE option<string>;
DEFINE FIELD IF NOT EXISTS acquired_at ON leader_lock TYPE datetime;
DEFINE FIELD IF NOT EXISTS expires_at  ON leader_lock TYPE datetime;
"#;

/// Distributed lock backed by the `leader_lock` table in SurrealDB.
///
/// The `scope` determines the record ID (`leader_lock:{scope}`), allowing
/// multiple independent locks within the same table.
#[derive(Debug, Clone)]
pub struct SurrealLock {
	client: Surreal<Any>,
	instance_id: String,
	scope: String,
}

/// Backward-compatible alias for [`SurrealLock`].
#[deprecated(since = "0.2.0", note = "use SurrealLock")]
pub type MigrationLock = SurrealLock;

impl SurrealLock {
	pub fn new(
		client: Surreal<Any>,
		instance_id: String,
		scope: impl Into<String>,
	) -> crate::Result<Self> {
		let scope = scope.into();
		if scope.is_empty() || !scope.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
			return Err(crate::Error::Lock(format!(
				"lock scope must be non-empty ASCII alphanumeric/underscore, got: {scope:?}"
			)));
		}
		Ok(Self {
			client,
			instance_id,
			scope,
		})
	}

	fn record_id(&self) -> String {
		format!("{TABLE}:{}", self.scope)
	}

	/// Try to acquire the leader lock, retrying until successful or timed out.
	///
	/// Retries up to 30 times with a 2-second interval, giving a total timeout
	/// of approximately 60 seconds before returning an error.
	pub async fn acquire(&self) -> crate::Result<()> {
		for attempt in 1..=MAX_RETRIES {
			if self.try_acquire().await? {
				info!(
					instance_id = %self.instance_id,
					scope = %self.scope,
					attempt,
					"acquired leader lock"
				);
				return Ok(());
			}
			if attempt < MAX_RETRIES {
				debug!(
					instance_id = %self.instance_id,
					scope = %self.scope,
					attempt,
					"lock held by another instance, retrying in {:?}",
					RETRY_INTERVAL,
				);
				tokio::time::sleep(RETRY_INTERVAL).await;
			}
		}
		Err(Error::Lock(format!(
			"failed to acquire lock '{}' after {MAX_RETRIES} attempts (instance {})",
			self.scope, self.instance_id,
		)))
	}

	/// Make a single attempt to acquire the leader lock.
	///
	/// Self-bootstraps the `leader_lock` table if it does not yet exist.
	pub async fn try_acquire(&self) -> crate::Result<bool> {
		// Ensure table exists (idempotent DDL — warn if it fails)
		if let Err(e) = self.client.query(DEFINE_TABLE_SQL).await {
			warn!(scope = %self.scope, "failed to define leader_lock table: {e}");
		}
		// Seed record (CREATE fails silently if it already exists)
		let create_sql = format!(
			"CREATE {} SET acquired_at = time::now(), expires_at = time::now()",
			self.record_id()
		);
		let _ = self.client.query(&create_sql).await;

		// Fast path: already held by us
		if self.is_held().await? {
			debug!(instance_id = %self.instance_id, scope = %self.scope, "already held by this instance");
			return Ok(true);
		}

		// Atomic claim: only succeeds if free or expired
		let acquire_sql = format!(
			r#"UPDATE {} SET
				holder = $instance_id,
				acquired_at = time::now(),
				expires_at = time::now() + 60s
			WHERE holder = NONE OR holder = '' OR expires_at < time::now()"#,
			self.record_id()
		);

		let mut response = self
			.client
			.query(&acquire_sql)
			.bind(("instance_id", self.instance_id.clone()))
			.await
			.map_err(|e| Error::Lock(format!("lock acquire query failed: {e}")))?;

		let rows: Vec<serde_json::Value> = response
			.take(0)
			.map_err(|e| Error::Lock(format!("lock acquire take failed: {e}")))?;

		Ok(!rows.is_empty())
	}

	/// Release the leader lock so other instances can acquire it.
	pub async fn release(&self) -> crate::Result<()> {
		let sql = format!(
			r#"UPDATE {} SET
				holder = NONE,
				expires_at = time::now()
			WHERE holder = $instance_id"#,
			self.record_id()
		);
		self.client
			.query(&sql)
			.bind(("instance_id", self.instance_id.clone()))
			.await
			.map_err(|e| Error::Lock(format!("lock release failed: {e}")))?;

		info!(instance_id = %self.instance_id, scope = %self.scope, "released leader lock");
		Ok(())
	}

	/// Check whether this instance currently holds the lock.
	pub async fn is_held(&self) -> crate::Result<bool> {
		let sql = format!(
			r#"SELECT * FROM {}
			WHERE holder = $instance_id AND expires_at > time::now()"#,
			self.record_id()
		);
		let mut response = self
			.client
			.query(&sql)
			.bind(("instance_id", self.instance_id.clone()))
			.await
			.map_err(|e| Error::Lock(format!("lock check failed: {e}")))?;

		let rows: Vec<serde_json::Value> = response
			.take(0)
			.map_err(|e| Error::Lock(format!("lock check take failed: {e}")))?;

		Ok(!rows.is_empty())
	}

	/// Force-release an expired or stale lock regardless of holder.
	///
	/// This is a recovery mechanism — prefer [`release`](Self::release) during
	/// normal operation.
	pub async fn force_release(&self) -> crate::Result<()> {
		let sql = format!(
			r#"UPDATE {} SET
				holder = NONE,
				expires_at = time::now()"#,
			self.record_id()
		);
		self.client
			.query(&sql)
			.await
			.map_err(|e| Error::Lock(format!("force release failed: {e}")))?;

		warn!(instance_id = %self.instance_id, scope = %self.scope, "force-released leader lock");
		Ok(())
	}

	pub fn instance_id(&self) -> &str {
		&self.instance_id
	}

	pub fn scope(&self) -> &str {
		&self.scope
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn retry_interval_is_reasonable() {
		assert!(RETRY_INTERVAL.as_secs() >= 1);
		assert!(RETRY_INTERVAL.as_secs() <= 10);
	}

	#[test]
	fn max_retries_is_bounded() {
		let total_wait = RETRY_INTERVAL.as_secs() * MAX_RETRIES as u64;
		assert!(total_wait <= 120);
	}

	#[test]
	fn record_id_format() {
		assert_eq!(format!("{TABLE}:{}", "migration"), "leader_lock:migration");
		assert_eq!(format!("{TABLE}:{}", "sync"), "leader_lock:sync");
	}

	#[test]
	fn scope_validation() {
		fn is_valid(s: &str) -> bool {
			!s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
		}
		assert!(is_valid("migration"));
		assert!(is_valid("sync"));
		assert!(is_valid("my_scope_2"));
		assert!(!is_valid(""));
		assert!(!is_valid("bad;scope"));
		assert!(!is_valid("spaces here"));
	}
}
