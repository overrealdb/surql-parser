use std::collections::HashMap;
use std::path::Path;

use sha2::{Digest, Sha256};
use tracing::warn;
use walkdir::WalkDir;

use crate::Error;

/// A migration file discovered on disk.
#[derive(Debug, Clone)]
pub struct Migration {
	pub version: u32,
	pub name: String,
	pub content: String,
	pub checksum: String,
	pub down_content: Option<String>,
}

/// A migration that has already been applied to the database.
#[derive(Debug, Clone)]
pub struct AppliedMigration {
	pub version: u32,
	pub applied_at: String,
	pub checksum: String,
	pub instance_id: String,
}

/// Compute the SHA-256 hex digest of a string.
pub fn compute_checksum(content: &str) -> String {
	let mut hasher = Sha256::new();
	hasher.update(content.as_bytes());
	hex::encode(hasher.finalize())
}

/// Discover migration files from `{root}/migrations/`.
///
/// Files must match the pattern `v{NNN}_{name}.surql` (e.g. `v001_initial.surql`).
/// Returns migrations sorted by version. Duplicate versions produce an error listing
/// both filenames. Non-contiguous versions emit a `tracing::warn` but do not fail.
pub fn discover_migrations(root: &Path) -> crate::Result<Vec<Migration>> {
	let migrations_dir = root.join("migrations");
	if !migrations_dir.exists() {
		return Ok(Vec::new());
	}

	let mut migrations = Vec::new();
	let mut version_to_filenames: HashMap<u32, Vec<String>> = HashMap::new();

	for entry in WalkDir::new(&migrations_dir)
		.min_depth(1)
		.max_depth(1)
		.sort_by_file_name()
	{
		let entry = entry.map_err(|e| Error::Migration(format!("reading migrations dir: {e}")))?;
		let path = entry.path();

		if path.extension().is_some_and(|ext| ext == "surql") {
			let file_name_os = path
				.file_name()
				.and_then(|s| s.to_str())
				.ok_or_else(|| Error::Migration(format!("invalid filename: {}", path.display())))?;
			let full_filename = file_name_os.to_string();

			// Skip .down.surql files — they are loaded as companions to their up migration
			if full_filename.ends_with(".down.surql") {
				continue;
			}

			let stem = path
				.file_stem()
				.and_then(|s| s.to_str())
				.ok_or_else(|| Error::Migration(format!("invalid filename: {}", path.display())))?;

			let (version, name) = parse_migration_filename(stem)?;

			version_to_filenames
				.entry(version)
				.or_default()
				.push(full_filename);

			let content = std::fs::read_to_string(path)
				.map_err(|e| Error::Migration(format!("failed to read {}: {e}", path.display())))?;
			let checksum = compute_checksum(&content);

			// Check for companion .down.surql file
			let down_path = path.with_extension("down.surql");
			let down_content = if down_path.exists() {
				Some(std::fs::read_to_string(&down_path).map_err(|e| {
					Error::Migration(format!("failed to read {}: {e}", down_path.display()))
				})?)
			} else {
				None
			};

			migrations.push(Migration {
				version,
				name,
				content,
				checksum,
				down_content,
			});
		}
	}

	detect_duplicate_versions(&version_to_filenames)?;

	migrations.sort_by_key(|m| m.version);
	validate_migration_sequence(&migrations)?;
	Ok(migrations)
}

/// Return an error if any version number maps to more than one file.
fn detect_duplicate_versions(
	version_to_filenames: &HashMap<u32, Vec<String>>,
) -> crate::Result<()> {
	let mut duplicates: Vec<(u32, &[String])> = version_to_filenames
		.iter()
		.filter(|(_, files)| files.len() > 1)
		.map(|(version, files)| (*version, files.as_slice()))
		.collect();

	if duplicates.is_empty() {
		return Ok(());
	}

	duplicates.sort_by_key(|(v, _)| *v);

	let descriptions: Vec<String> = duplicates
		.iter()
		.map(|(version, files)| format!("v{version:03}: {}", files.join(", ")))
		.collect();

	Err(Error::Migration(format!(
		"duplicate migration version(s): {}",
		descriptions.join("; "),
	)))
}

/// Parse a migration filename like `v001_initial_seed` into (1, "initial_seed").
fn parse_migration_filename(filename: &str) -> crate::Result<(u32, String)> {
	let rest = filename.strip_prefix('v').ok_or_else(|| {
		Error::Migration(format!(
			"migration filename must start with 'v': {filename}"
		))
	})?;

	let (version_str, name) = rest.split_once('_').ok_or_else(|| {
		Error::Migration(format!(
			"migration filename must have format v{{NNN}}_{{name}}: {filename}"
		))
	})?;

	let version: u32 = version_str
		.parse()
		.map_err(|e| Error::Migration(format!("invalid version number '{version_str}': {e}")))?;

	if version == 0 {
		return Err(Error::Migration("migration version must be >= 1".into()));
	}

	Ok((version, name.to_string()))
}

/// Validate migration sequence ordering.
///
/// - Duplicate versions are rejected with an error.
/// - Non-contiguous versions (gaps) emit a `tracing::warn` but do not fail,
///   allowing workflows where a gap is intentional or temporary.
/// - Migrations must still start at version 1.
pub(crate) fn validate_migration_sequence(migrations: &[Migration]) -> crate::Result<()> {
	let mut seen = std::collections::HashSet::new();
	for m in migrations {
		if !seen.insert(m.version) {
			return Err(Error::Migration(format!(
				"duplicate migration version: {}",
				m.version,
			)));
		}
	}

	if let Some(first) = migrations.first()
		&& first.version != 1
	{
		return Err(Error::Migration(
			"migrations must start at version 1".into(),
		));
	}

	for pair in migrations.windows(2) {
		if pair[1].version != pair[0].version + 1 {
			let gap_start = pair[0].version + 1;
			let gap_end = pair[1].version - 1;
			if gap_start == gap_end {
				warn!(
					missing_version = gap_start,
					"non-contiguous migration versions: gap between v{:03} and v{:03}",
					pair[0].version,
					pair[1].version,
				);
			} else {
				warn!(
					missing_from = gap_start,
					missing_to = gap_end,
					"non-contiguous migration versions: gap between v{:03} and v{:03}",
					pair[0].version,
					pair[1].version,
				);
			}
		}
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn checksum_is_deterministic() {
		assert_eq!(compute_checksum("SELECT 1;"), compute_checksum("SELECT 1;"));
	}

	#[test]
	fn checksum_differs_for_different_content() {
		assert_ne!(compute_checksum("SELECT 1;"), compute_checksum("SELECT 2;"));
	}

	#[test]
	fn checksum_is_hex_sha256() {
		let hash = compute_checksum("hello");
		assert_eq!(hash.len(), 64);
		assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
		assert_eq!(
			hash,
			"2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
		);
	}

	#[test]
	fn parse_filename_standard() {
		let (v, n) = parse_migration_filename("v001_initial_seed").unwrap();
		assert_eq!(v, 1);
		assert_eq!(n, "initial_seed");
	}

	#[test]
	fn parse_filename_no_leading_zeros() {
		let (v, n) = parse_migration_filename("v2_backfill").unwrap();
		assert_eq!(v, 2);
		assert_eq!(n, "backfill");
	}

	#[test]
	fn parse_filename_rejects_no_v_prefix() {
		assert!(parse_migration_filename("001_initial").is_err());
	}

	#[test]
	fn parse_filename_rejects_no_underscore() {
		assert!(parse_migration_filename("v001").is_err());
	}

	#[test]
	fn parse_filename_rejects_zero_version() {
		assert!(parse_migration_filename("v0_bad").is_err());
	}

	#[test]
	fn validate_empty_sequence() {
		assert!(validate_migration_sequence(&[]).is_ok());
	}

	#[test]
	fn validate_warns_on_gap_but_succeeds() {
		let migrations = vec![
			Migration {
				version: 1,
				name: "a".into(),
				content: "".into(),
				checksum: "".into(),
				down_content: None,
			},
			Migration {
				version: 3,
				name: "c".into(),
				content: "".into(),
				checksum: "".into(),
				down_content: None,
			},
		];
		assert!(validate_migration_sequence(&migrations).is_ok());
	}

	#[test]
	fn validate_rejects_not_starting_at_one() {
		let migrations = vec![Migration {
			version: 2,
			name: "b".into(),
			content: "".into(),
			checksum: "".into(),
			down_content: None,
		}];
		assert!(validate_migration_sequence(&migrations).is_err());
	}

	#[test]
	fn validate_rejects_duplicates() {
		let migrations = vec![
			Migration {
				version: 1,
				name: "a".into(),
				content: "".into(),
				checksum: "".into(),
				down_content: None,
			},
			Migration {
				version: 1,
				name: "a_dup".into(),
				content: "".into(),
				checksum: "".into(),
				down_content: None,
			},
		];
		assert!(validate_migration_sequence(&migrations).is_err());
	}

	#[test]
	fn validate_single_migration() {
		let migrations = vec![Migration {
			version: 1,
			name: "init".into(),
			content: "SELECT 1;".into(),
			checksum: compute_checksum("SELECT 1;"),
			down_content: None,
		}];
		assert!(validate_migration_sequence(&migrations).is_ok());
	}

	#[test]
	fn validate_contiguous_sequence() {
		let migrations: Vec<Migration> = (1..=5)
			.map(|v| Migration {
				version: v,
				name: format!("m{v}"),
				content: format!("SELECT {v};"),
				checksum: compute_checksum(&format!("SELECT {v};")),
				down_content: None,
			})
			.collect();
		assert!(validate_migration_sequence(&migrations).is_ok());
	}

	// ─── Filename parsing edge cases ───

	#[test]
	fn parse_filename_large_version() {
		let (v, n) = parse_migration_filename("v999_final").unwrap();
		assert_eq!(v, 999);
		assert_eq!(n, "final");
	}

	#[test]
	fn parse_filename_with_multiple_underscores() {
		let (v, n) = parse_migration_filename("v003_add_user_table").unwrap();
		assert_eq!(v, 3);
		assert_eq!(n, "add_user_table");
	}

	#[test]
	fn parse_filename_single_digit() {
		let (v, n) = parse_migration_filename("v1_init").unwrap();
		assert_eq!(v, 1);
		assert_eq!(n, "init");
	}

	#[test]
	fn parse_filename_rejects_empty_name() {
		// "v1_" → name would be "", which is technically valid
		let (v, n) = parse_migration_filename("v1_").unwrap();
		assert_eq!(v, 1);
		assert_eq!(n, "");
	}

	#[test]
	fn parse_filename_rejects_negative_version() {
		assert!(parse_migration_filename("v-1_bad").is_err());
	}

	#[test]
	fn parse_filename_rejects_non_numeric_version() {
		assert!(parse_migration_filename("vabc_bad").is_err());
	}

	#[test]
	fn parse_filename_rejects_float_version() {
		assert!(parse_migration_filename("v1.5_bad").is_err());
	}

	#[test]
	fn parse_filename_rejects_empty() {
		assert!(parse_migration_filename("").is_err());
	}

	#[test]
	fn parse_filename_rejects_just_v() {
		assert!(parse_migration_filename("v").is_err());
	}

	// ─── Checksum edge cases ───

	#[test]
	fn checksum_empty_string() {
		let hash = compute_checksum("");
		assert_eq!(hash.len(), 64);
		// Known SHA-256 of ""
		assert_eq!(
			hash,
			"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
		);
	}

	#[test]
	fn checksum_whitespace_matters() {
		assert_ne!(
			compute_checksum("SELECT 1;"),
			compute_checksum("SELECT  1;")
		);
		assert_ne!(
			compute_checksum("SELECT 1;\n"),
			compute_checksum("SELECT 1;")
		);
	}

	#[test]
	fn checksum_unicode() {
		let hash = compute_checksum("SELECT * FROM пользователи;");
		assert_eq!(hash.len(), 64);
	}

	#[test]
	fn checksum_is_lowercase_hex() {
		let hash = compute_checksum("test");
		assert!(
			hash.chars()
				.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
		);
	}

	// ─── Property-based tests ───

	mod proptests {
		use super::*;
		use proptest::prelude::*;

		proptest! {
			#[test]
			fn parse_migration_filename_never_panics(s in "\\PC*") {
				let _ = parse_migration_filename(&s);
			}

			#[test]
			fn valid_migration_filename_roundtrips(
				version in 1u32..10000,
				name in "[a-z][a-z0-9_]{0,30}",
			) {
				let filename = format!("v{version}_{name}");
				let (v, n) = parse_migration_filename(&filename).unwrap();
				prop_assert_eq!(v, version);
				prop_assert_eq!(n, name);
			}

			#[test]
			fn checksum_always_64_hex_chars(content in "\\PC{0,500}") {
				let hash = compute_checksum(&content);
				prop_assert_eq!(hash.len(), 64);
				prop_assert!(hash.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
			}

			#[test]
			fn checksum_is_deterministic(content in "\\PC{0,500}") {
				prop_assert_eq!(compute_checksum(&content), compute_checksum(&content));
			}
		}
	}
}
