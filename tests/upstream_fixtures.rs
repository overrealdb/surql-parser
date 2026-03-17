//! Tests imported from SurrealDB language-tests/tests/parsing/.
//!
//! These .surql files are copied from upstream by sync-upstream.sh.
//! Each file contains SurrealQL statements (after the /** ... */ header comment).
//! Files in `error/` subdirectories are expected to FAIL parsing.
//!
//! Source: https://github.com/surrealdb/surrealdb/tree/main/language-tests/tests/parsing

use std::path::Path;
use surql_parser::parse;

/// Extract the SurrealQL body from a test file (everything after the `*/` closing comment).
fn extract_surql(content: &str) -> &str {
	if let Some(pos) = content.find("*/") {
		content[pos + 2..].trim()
	} else {
		content.trim()
	}
}

/// Check if a test file is expected to produce a parse error.
fn is_error_test(path: &Path) -> bool {
	// Check path components for "error" or "errors" directories
	path.components()
		.any(|c| c.as_os_str() == "error" || c.as_os_str() == "errors")
		// Also check the TOML header for "parsing-error"
		|| std::fs::read_to_string(path)
			.map(|c| c.contains("parsing-error"))
			.unwrap_or(false)
}

/// Run all .surql fixtures from the given directory.
fn run_fixtures(dir: &str) {
	let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
		.join("tests/fixtures/parsing")
		.join(dir);

	if !fixture_dir.exists() {
		eprintln!("Fixture directory not found: {}", fixture_dir.display());
		return;
	}

	let mut passed = 0;
	let mut failed = 0;
	let mut skipped = 0;

	for entry in walkdir::WalkDir::new(&fixture_dir)
		.into_iter()
		.filter_map(|e| e.ok())
		.filter(|e| e.path().extension().is_some_and(|ext| ext == "surql"))
	{
		let path = entry.path();
		let content = std::fs::read_to_string(path).unwrap();
		let surql = extract_surql(&content);

		if surql.is_empty() {
			skipped += 1;
			continue;
		}

		let rel = path.strip_prefix(&fixture_dir).unwrap_or(path);
		let result = parse(surql);

		if is_error_test(path) {
			// Error tests should fail
			if result.is_err() {
				passed += 1;
			} else {
				eprintln!("UNEXPECTED PASS (expected error): {}", rel.display());
				failed += 1;
			}
		} else {
			// Normal tests should succeed
			match result {
				Ok(_) => passed += 1,
				Err(e) => {
					eprintln!("PARSE FAILED: {} — {e}", rel.display());
					failed += 1;
				}
			}
		}
	}

	eprintln!("  {dir}: {passed} passed, {failed} failed, {skipped} skipped");
	assert_eq!(failed, 0, "{failed} fixture(s) failed in {dir}/");
}

#[test]
fn fixtures_strings() {
	run_fixtures("strings");
}

#[test]
fn fixtures_basic() {
	run_fixtures("basic");
}

#[test]
fn fixtures_statements() {
	run_fixtures("statements");
}

#[test]
fn fixtures_expr() {
	run_fixtures("expr");
}

#[test]
fn fixtures_datetime() {
	run_fixtures("datetime");
}

#[test]
fn fixtures_idents() {
	run_fixtures("idents");
}

#[test]
fn fixtures_bytes() {
	run_fixtures("bytes");
}

#[test]
fn fixtures_recordid_string() {
	run_fixtures("recordid_string");
}

#[test]
fn fixtures_errors() {
	run_fixtures("errors");
}

#[test]
fn fixtures_far_peeking() {
	run_fixtures("far_peeking");
}

#[test]
fn fixtures_file() {
	run_fixtures("file");
}

#[test]
fn fixtures_glueing() {
	run_fixtures("glueing");
}

#[test]
fn fixtures_deprecate() {
	run_fixtures("deprecate");
}
