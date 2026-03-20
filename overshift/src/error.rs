use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("manifest: {0}")]
	Manifest(String),

	#[error("migration: {0}")]
	Migration(String),

	#[error("schema: {0}")]
	Schema(String),

	#[error("lock: {0}")]
	Lock(String),

	#[error("validation: {0}")]
	Validation(String),

	#[error("snapshot: {0}")]
	Snapshot(String),

	#[error("checksum mismatch for v{version:03}_{name}: expected {expected}, found {actual}")]
	ChecksumMismatch {
		version: u32,
		name: String,
		expected: String,
		actual: String,
	},

	#[error("database: {0}")]
	Database(#[from] surrealdb::Error),

	#[error("io: {0}")]
	Io(#[from] std::io::Error),

	#[error("toml: {0}")]
	Toml(#[from] toml::de::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn display_checksum_mismatch() {
		let e = Error::ChecksumMismatch {
			version: 3,
			name: "seed".into(),
			expected: "aaa".into(),
			actual: "bbb".into(),
		};
		assert_eq!(
			e.to_string(),
			"checksum mismatch for v003_seed: expected aaa, found bbb"
		);
	}

	#[test]
	fn display_checksum_mismatch_formatting() {
		// Version 1 should be formatted as v001
		let e = Error::ChecksumMismatch {
			version: 1,
			name: "init".into(),
			expected: "x".into(),
			actual: "y".into(),
		};
		assert!(e.to_string().contains("v001_init"));
	}

	#[test]
	fn io_error_converts() {
		let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
		let e: Error = io_err.into();
		assert!(e.to_string().contains("gone"));
	}

	#[test]
	fn error_is_send_sync() {
		fn assert_send_sync<T: Send + Sync>() {}
		assert_send_sync::<Error>();
	}
}
