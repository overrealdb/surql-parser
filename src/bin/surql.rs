//! surql — CLI tool for SurrealQL files.
//!
//! Install: cargo install surql-parser --features cli
//!
//! Usage:
//!   surql check schema/**/*.surql     — validate files parse correctly
//!   surql fmt file.surql              — format SurrealQL
//!   surql info schema/                — show schema summary
//!   surql diff schema/                — show uncommitted schema changes (vs HEAD)
//!   surql diff --before v1/ --after v2/   — compare two schema directories
//!   surql docs schema/               — generate markdown docs from .surql files
//!   surql docs schema/ --output docs.md  — write docs to file
//!   surql lint schema/               — run SurrealQL-specific lints
//!   surql lint schema/ --fix         — auto-fix fixable lint issues
//!   surql test tests/                — run .surql test files against SurrealDB

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "surql", about = "SurrealQL file toolkit", version)]
struct Cli {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
enum Command {
	/// Validate SurrealQL files parse correctly
	Check {
		/// Files or directories to check (recursive)
		#[arg(required = true)]
		paths: Vec<PathBuf>,
		/// Also warn about undefined table references (needs schema graph)
		#[arg(long)]
		strict: bool,
	},
	/// Format SurrealQL files in-place
	Fmt {
		/// Files or directories to format
		#[arg(required = true)]
		paths: Vec<PathBuf>,
		/// Don't modify files, exit 1 if formatting changes needed (for CI)
		#[arg(long)]
		check: bool,
		/// Show unified diff of formatting changes
		#[arg(long)]
		diff: bool,
	},
	/// Show schema summary (tables, fields, functions)
	Info {
		/// Files or directories to analyze
		#[arg(required = true)]
		paths: Vec<PathBuf>,
	},
	/// Compare schema between two states and show what changed
	Diff {
		/// Directory with .surql files (current state, used for git diff when no --before)
		#[arg(required = true)]
		path: PathBuf,
		/// "Before" state: directory or file (if omitted, uses git HEAD)
		#[arg(long)]
		before: Option<PathBuf>,
		/// "After" state: directory or file (defaults to <path>)
		#[arg(long)]
		after: Option<PathBuf>,
	},
	/// Generate schema documentation from COMMENT fields
	Docs {
		/// Files or directories to document
		#[arg(required = true)]
		path: PathBuf,
		/// Output format (currently only "md" is supported)
		#[arg(long, default_value = "md")]
		format: String,
		/// Write output to file instead of stdout
		#[arg(long, short)]
		output: Option<PathBuf>,
	},
	/// Run SurrealQL-specific lints on schema files
	Lint {
		/// Files or directories to lint
		#[arg(required = true)]
		paths: Vec<PathBuf>,
		/// Auto-fix fixable lint issues (e.g., add TYPE any to untyped fields)
		#[arg(long)]
		fix: bool,
	},
	/// Run .surql test files (execute and check for errors)
	Test {
		/// Directory containing .surql test files (default: tests/)
		#[arg(long, default_value = "tests")]
		path: PathBuf,
		/// Schema directory to load before running tests
		#[arg(long)]
		schema: Option<PathBuf>,
	},
}

fn collect_surql_files(paths: &[PathBuf]) -> Vec<PathBuf> {
	let mut files = Vec::new();
	for path in paths {
		if path.is_file() {
			files.push(path.clone());
		} else if path.is_dir() {
			for entry in walkdir::WalkDir::new(path)
				.into_iter()
				.filter_map(|e| e.ok())
				.filter(|e| e.path().extension().is_some_and(|ext| ext == "surql"))
			{
				files.push(entry.into_path());
			}
		} else {
			eprintln!("Path not found: {}", path.display());
		}
	}
	files.sort();
	files
}

fn read_file(path: &PathBuf) -> Result<String, String> {
	std::fs::read_to_string(path).map_err(|e| format!("Error reading {}: {e}", path.display()))
}

fn main() -> ExitCode {
	let cli = Cli::parse();

	match cli.command {
		Command::Check { paths, strict } => cmd_check(&paths, strict),
		Command::Fmt { paths, check, diff } => cmd_fmt(&paths, check, diff),
		Command::Info { paths } => cmd_info(&paths),
		Command::Diff {
			path,
			before,
			after,
		} => cmd_diff(&path, before.as_deref(), after.as_deref()),
		Command::Docs {
			path,
			format,
			output,
		} => cmd_docs(&path, &format, output.as_deref()),
		Command::Lint { paths, fix } => cmd_lint(&paths, fix),
		Command::Test { path, schema } => cmd_test(&path, schema.as_deref()),
	}
}

// ─── check ───

fn cmd_check(paths: &[PathBuf], strict: bool) -> ExitCode {
	let files = collect_surql_files(paths);
	if files.is_empty() {
		eprintln!("No .surql files found.");
		return ExitCode::FAILURE;
	}

	let mut passed = 0u32;
	let mut failed = 0u32;

	for file in &files {
		let content = match read_file(file) {
			Ok(c) => c,
			Err(e) => {
				eprintln!("{e}");
				failed += 1;
				continue;
			}
		};

		match surql_parser::parse_for_diagnostics(&content) {
			Ok(_) => {
				passed += 1;
			}
			Err(diags) => {
				for d in &diags {
					eprintln!("{}:{}:{}: {}", file.display(), d.line, d.column, d.message);
				}
				failed += 1;
			}
		}
	}

	if strict {
		let strict_warnings = check_strict(paths);
		if strict_warnings > 0 {
			eprintln!("{strict_warnings} strict warning(s)");
			failed += strict_warnings;
		}
	}

	eprintln!("{passed} passed, {failed} failed ({} files)", files.len());
	if failed > 0 {
		ExitCode::FAILURE
	} else {
		ExitCode::SUCCESS
	}
}

fn check_strict(paths: &[PathBuf]) -> u32 {
	let files = collect_surql_files(paths);
	let mut graph = surql_parser::SchemaGraph::default();
	let mut all_sources = Vec::new();

	for file in &files {
		let content = match read_file(file) {
			Ok(c) => c,
			Err(_) => continue,
		};
		let (stmts, _) = surql_parser::parse_with_recovery(&content);
		if let Ok(defs) = surql_parser::extract_definitions_from_ast(&stmts) {
			let file_graph = surql_parser::SchemaGraph::from_definitions(&defs);
			graph.merge(file_graph);
		}
		all_sources.push((file.clone(), content));
	}

	let defined_tables: std::collections::HashSet<String> =
		graph.table_names().map(|s| s.to_string()).collect();

	let mut warnings = 0u32;
	for (file, content) in &all_sources {
		let (stmts, _) = surql_parser::parse_with_recovery(content);
		for table_ref in extract_table_references(&stmts) {
			if !defined_tables.contains(&table_ref) {
				eprintln!(
					"{}:0:0: strict: table '{}' referenced but not defined",
					file.display(),
					table_ref
				);
				warnings += 1;
			}
		}
	}

	warnings
}

fn extract_table_references(
	stmts: &[surql_parser::upstream::sql::ast::TopLevelExpr],
) -> Vec<String> {
	use surql_parser::upstream::sql::ast::TopLevelExpr;

	let mut refs = Vec::new();
	for top in stmts {
		if let TopLevelExpr::Expr(expr) = top {
			collect_table_refs_from_expr(expr, &mut refs);
		}
	}
	refs
}

fn collect_table_refs_from_expr(expr: &surql_parser::Expr, refs: &mut Vec<String>) {
	use surql_parser::Expr;
	use surrealdb_types::{SqlFormat, ToSql};

	fn exprs_to_names(exprs: &[surql_parser::Expr], refs: &mut Vec<String>) {
		for what in exprs {
			let mut name = String::new();
			ToSql::fmt_sql(what, &mut name, SqlFormat::SingleLine);
			let name = name
				.trim_matches('`')
				.trim_matches('\u{27E8}')
				.trim_matches('\u{27E9}')
				.to_string();
			if !name.contains('(') && !name.contains('*') && !name.is_empty() {
				refs.push(name);
			}
		}
	}

	match expr {
		Expr::Select(s) => exprs_to_names(&s.what, refs),
		Expr::Create(s) => exprs_to_names(&s.what, refs),
		Expr::Update(s) => exprs_to_names(&s.what, refs),
		Expr::Delete(s) => exprs_to_names(&s.what, refs),
		_ => {}
	}
}

// ─── fmt ───

fn cmd_fmt(paths: &[PathBuf], check_only: bool, show_diff: bool) -> ExitCode {
	let files = collect_surql_files(paths);
	if files.is_empty() {
		eprintln!("No .surql files found.");
		return ExitCode::FAILURE;
	}

	let cwd = std::env::current_dir().unwrap_or_else(|e| {
		eprintln!("Warning: cannot determine working directory: {e}, using '.'");
		PathBuf::from(".")
	});
	let config = surql_parser::formatting::FormatConfig::load_from_dir(&cwd);

	let mut unformatted = 0u32;
	let mut formatted_count = 0u32;
	let mut error_count = 0u32;

	for file in &files {
		let content = match read_file(file) {
			Ok(c) => c,
			Err(e) => {
				eprintln!("{e}");
				error_count += 1;
				continue;
			}
		};

		match surql_parser::formatting::format_source(&content, &config) {
			Some(formatted) => {
				if check_only {
					eprintln!("Would reformat: {}", file.display());
					if show_diff {
						print_diff(file, &content, &formatted);
					}
					unformatted += 1;
				} else {
					if show_diff {
						print_diff(file, &content, &formatted);
					}
					match std::fs::write(file, &formatted) {
						Ok(()) => {
							eprintln!("Formatted: {}", file.display());
							formatted_count += 1;
						}
						Err(e) => {
							eprintln!("Error writing {}: {e}", file.display());
							error_count += 1;
						}
					}
				}
			}
			None => {
				// format_source returns None for both "already formatted" and "lexer error".
				// Distinguish by checking if the file can be parsed at all.
				if surql_parser::parse(&content).is_err() {
					eprintln!("Skipped {}: parse error", file.display());
					error_count += 1;
				}
			}
		}
	}

	if check_only {
		if unformatted > 0 {
			eprintln!(
				"{unformatted} file(s) would be reformatted ({} checked)",
				files.len()
			);
			return ExitCode::FAILURE;
		}
		eprintln!("All {} file(s) formatted correctly.", files.len());
		return ExitCode::SUCCESS;
	}

	if error_count > 0 {
		eprintln!(
			"{formatted_count} formatted, {error_count} errors ({} files)",
			files.len()
		);
		return ExitCode::FAILURE;
	}

	if formatted_count > 0 {
		eprintln!(
			"{formatted_count} file(s) formatted ({} checked)",
			files.len()
		);
	} else {
		eprintln!("All {} file(s) already formatted.", files.len());
	}
	ExitCode::SUCCESS
}

fn print_diff(file: &std::path::Path, original: &str, formatted: &str) {
	let orig_lines: Vec<&str> = original.lines().collect();
	let new_lines: Vec<&str> = formatted.lines().collect();

	eprintln!("--- {}", file.display());
	eprintln!("+++ {}", file.display());

	let max = orig_lines.len().max(new_lines.len());
	let mut i = 0;
	while i < max {
		let orig = orig_lines.get(i).copied().unwrap_or("");
		let new = new_lines.get(i).copied().unwrap_or("");
		if orig != new {
			let hunk_start = i;
			let mut hunk_end = i;
			while hunk_end < max {
				let o = orig_lines.get(hunk_end).copied().unwrap_or("");
				let n = new_lines.get(hunk_end).copied().unwrap_or("");
				if o == n {
					break;
				}
				hunk_end += 1;
			}
			eprintln!(
				"@@ -{},{} +{},{} @@",
				hunk_start + 1,
				hunk_end - hunk_start,
				hunk_start + 1,
				hunk_end - hunk_start
			);
			for j in hunk_start..hunk_end {
				if let Some(o) = orig_lines.get(j) {
					eprintln!("-{o}");
				}
			}
			for j in hunk_start..hunk_end {
				if let Some(n) = new_lines.get(j) {
					eprintln!("+{n}");
				}
			}
			i = hunk_end;
		} else {
			i += 1;
		}
	}
}

// ─── info ───

fn cmd_info(paths: &[PathBuf]) -> ExitCode {
	let files = collect_surql_files(paths);
	if files.is_empty() {
		eprintln!("No .surql files found.");
		return ExitCode::FAILURE;
	}

	let mut graph = surql_parser::SchemaGraph::default();
	let mut parse_errors = 0u32;

	for file in &files {
		let content = match read_file(file) {
			Ok(c) => c,
			Err(e) => {
				eprintln!("{e}");
				continue;
			}
		};
		let (stmts, _) = surql_parser::parse_with_recovery(&content);
		match surql_parser::extract_definitions_from_ast(&stmts) {
			Ok(defs) => {
				let file_graph = surql_parser::SchemaGraph::from_definitions(&defs);
				graph.merge(file_graph);
			}
			Err(e) => {
				eprintln!("Error parsing {}: {e}", file.display());
				parse_errors += 1;
			}
		}
	}

	let table_names: Vec<&str> = graph.table_names().collect();
	let table_count = table_names.len();
	let mut total_fields = 0usize;
	let mut total_indexes = 0usize;
	let mut total_events = 0usize;
	for name in &table_names {
		total_fields += graph.fields_of(name).len();
		total_indexes += graph.indexes_of(name).len();
		total_events += graph.events_of(name).len();
	}
	let function_count = graph.function_names().count();
	let param_count = graph.param_names().count();

	println!("Schema Summary ({} files)", files.len());
	println!("  Tables:    {table_count}");
	println!("  Fields:    {total_fields}");
	println!("  Indexes:   {total_indexes}");
	println!("  Events:    {total_events}");
	println!("  Functions: {function_count}");
	println!("  Params:    {param_count}");

	if !table_names.is_empty() {
		println!();
		println!("Tables:");
		let mut sorted_names: Vec<&str> = table_names;
		sorted_names.sort();
		for name in &sorted_names {
			let fields = graph.fields_of(name);
			let indexes = graph.indexes_of(name);
			let schemafull = graph
				.table(name)
				.map(|t| if t.full { " SCHEMAFULL" } else { "" })
				.unwrap_or("");
			println!(
				"  {name}{schemafull} ({} fields, {} indexes)",
				fields.len(),
				indexes.len()
			);
		}
	}

	let function_names: Vec<&str> = graph.function_names().collect();
	if !function_names.is_empty() {
		println!();
		println!("Functions:");
		let mut sorted_fns: Vec<&str> = function_names;
		sorted_fns.sort();
		for name in &sorted_fns {
			if let Some(f) = graph.function(name) {
				let args_str: Vec<String> = f
					.args
					.iter()
					.map(|(name, kind)| {
						let n = name.strip_prefix('$').unwrap_or(name);
						format!("${n}: {kind}")
					})
					.collect();
				let ret = f
					.returns
					.as_deref()
					.map(|r| format!(" -> {r}"))
					.unwrap_or_else(|| " -> unknown".to_string());
				println!("  fn::{}({}){ret}", name, args_str.join(", "));
			}
		}
	}

	if parse_errors > 0 {
		eprintln!("{parse_errors} file(s) had parse errors.");
	}

	ExitCode::SUCCESS
}

// ─── docs ───

fn cmd_docs(path: &std::path::Path, format: &str, output: Option<&std::path::Path>) -> ExitCode {
	if format != "md" {
		eprintln!("Unsupported format: {format} (only 'md' is currently supported)");
		return ExitCode::FAILURE;
	}

	let graph = match build_graph_from_path(path) {
		Ok(g) => g,
		Err(e) => {
			eprintln!("{e}");
			return ExitCode::FAILURE;
		}
	};

	let markdown = graph.build_docs_markdown();

	if let Some(out_path) = output {
		match std::fs::write(out_path, &markdown) {
			Ok(()) => {
				eprintln!("Documentation written to {}", out_path.display());
				ExitCode::SUCCESS
			}
			Err(e) => {
				eprintln!("Error writing {}: {e}", out_path.display());
				ExitCode::FAILURE
			}
		}
	} else {
		print!("{markdown}");
		ExitCode::SUCCESS
	}
}

// ─── diff ───

fn cmd_diff(
	path: &std::path::Path,
	before: Option<&std::path::Path>,
	after: Option<&std::path::Path>,
) -> ExitCode {
	let after_path = after.unwrap_or(path);

	let after_graph = match build_graph_from_path(after_path) {
		Ok(g) => g,
		Err(e) => {
			eprintln!("Error reading 'after' schema: {e}");
			return ExitCode::FAILURE;
		}
	};

	let before_graph = if let Some(before_path) = before {
		match build_graph_from_path(before_path) {
			Ok(g) => g,
			Err(e) => {
				eprintln!("Error reading 'before' schema: {e}");
				return ExitCode::FAILURE;
			}
		}
	} else {
		match build_graph_from_git_head(path) {
			Ok(g) => g,
			Err(e) => {
				eprintln!("Error reading git HEAD schema: {e}");
				return ExitCode::FAILURE;
			}
		}
	};

	let diff = surql_parser::diff::compare_schemas(&before_graph, &after_graph);
	if diff.is_empty() {
		println!("No schema changes.");
	} else {
		print!("{diff}");
	}
	ExitCode::SUCCESS
}

fn build_graph_from_path(path: &std::path::Path) -> Result<surql_parser::SchemaGraph, String> {
	if path.is_file() {
		let content = std::fs::read_to_string(path)
			.map_err(|e| format!("Error reading {}: {e}", path.display()))?;
		surql_parser::SchemaGraph::from_source(&content)
			.map_err(|e| format!("Error parsing {}: {e}", path.display()))
	} else if path.is_dir() {
		surql_parser::SchemaGraph::from_files(path)
			.map_err(|e| format!("Error reading directory {}: {e}", path.display()))
	} else {
		Err(format!("Path not found: {}", path.display()))
	}
}

fn build_graph_from_git_head(dir: &std::path::Path) -> Result<surql_parser::SchemaGraph, String> {
	let files = collect_surql_files(&[dir.to_path_buf()]);
	if files.is_empty() {
		return Ok(surql_parser::SchemaGraph::default());
	}

	let repo_root = find_git_repo_root(dir)?;

	let mut graph = surql_parser::SchemaGraph::default();
	for file in &files {
		let relative = file.strip_prefix(&repo_root).map_err(|_| {
			format!(
				"File {} is not under git root {}",
				file.display(),
				repo_root.display()
			)
		})?;

		let git_path = relative.to_string_lossy().to_string();
		match git_show_head(&repo_root, &git_path) {
			Ok(content) => {
				let (stmts, _) = surql_parser::parse_with_recovery(&content);
				if let Ok(defs) = surql_parser::extract_definitions_from_ast(&stmts) {
					let file_graph = surql_parser::SchemaGraph::from_definitions(&defs);
					graph.merge(file_graph);
				}
			}
			Err(_) => {
				// File is new (not in HEAD) — treat as absent in "before"
			}
		}
	}
	Ok(graph)
}

fn find_git_repo_root(start: &std::path::Path) -> Result<PathBuf, String> {
	let output = std::process::Command::new("git")
		.args(["rev-parse", "--show-toplevel"])
		.current_dir(start)
		.output()
		.map_err(|e| format!("Failed to run git: {e}"))?;

	if !output.status.success() {
		return Err(format!(
			"git rev-parse failed: {}",
			String::from_utf8_lossy(&output.stderr).trim()
		));
	}

	let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
	Ok(PathBuf::from(root))
}

fn git_show_head(repo_root: &std::path::Path, relative_path: &str) -> Result<String, String> {
	let spec = format!("HEAD:{relative_path}");
	let output = std::process::Command::new("git")
		.args(["show", &spec])
		.current_dir(repo_root)
		.output()
		.map_err(|e| format!("Failed to run git show: {e}"))?;

	if !output.status.success() {
		return Err(format!(
			"git show {spec} failed: {}",
			String::from_utf8_lossy(&output.stderr).trim()
		));
	}

	String::from_utf8(output.stdout).map_err(|e| format!("git show output is not valid UTF-8: {e}"))
}

// ─── lint ───

fn cmd_lint(paths: &[PathBuf], fix: bool) -> ExitCode {
	let files = collect_surql_files(paths);
	if files.is_empty() {
		eprintln!("No .surql files found.");
		return ExitCode::FAILURE;
	}

	let mut graph = surql_parser::SchemaGraph::default();
	let mut sources: Vec<(PathBuf, String)> = Vec::new();

	for file in &files {
		let content = match read_file(file) {
			Ok(c) => c,
			Err(e) => {
				eprintln!("{e}");
				continue;
			}
		};
		let (stmts, _) = surql_parser::parse_with_recovery(&content);
		if let Ok(defs) = surql_parser::extract_definitions_from_ast(&stmts) {
			let file_graph = surql_parser::SchemaGraph::from_definitions(&defs);
			graph.merge(file_graph);
		}
		sources.push((file.clone(), content));
	}

	let results = surql_parser::lint::lint_schema(&graph, &sources);

	if results.is_empty() {
		eprintln!("No lint issues found ({} files checked).", files.len());
		return ExitCode::SUCCESS;
	}

	for result in &results {
		println!("{result}");
	}

	let warning_count = results
		.iter()
		.filter(|r| r.severity == surql_parser::lint::LintSeverity::Warning)
		.count();
	let info_count = results.len() - warning_count;
	eprintln!(
		"{} warning(s), {} info(s) in {} file(s)",
		warning_count,
		info_count,
		files.len()
	);

	if fix {
		let mut total_fixes = 0u32;
		for (path, content) in &sources {
			let (fixed, count) = surql_parser::lint::apply_fixes(content);
			if count > 0 {
				match std::fs::write(path, &fixed) {
					Ok(()) => {
						eprintln!("Fixed {} issue(s) in {}", count, path.display());
						total_fixes += count;
					}
					Err(e) => {
						eprintln!("Error writing {}: {e}", path.display());
					}
				}
			}
		}
		if total_fixes > 0 {
			eprintln!("{total_fixes} fix(es) applied.");
		} else {
			eprintln!("No auto-fixable issues found.");
		}
	}

	if warning_count > 0 {
		ExitCode::FAILURE
	} else {
		ExitCode::SUCCESS
	}
}

// ─── test ───

fn cmd_test(path: &std::path::Path, schema: Option<&std::path::Path>) -> ExitCode {
	if !path.is_dir() {
		eprintln!("Test directory not found: {}", path.display());
		return ExitCode::FAILURE;
	}

	let test_files = collect_surql_files(&[path.to_path_buf()]);
	if test_files.is_empty() {
		eprintln!("No .surql test files found in {}", path.display());
		return ExitCode::FAILURE;
	}

	let schema_content = if let Some(schema_path) = schema {
		let schema_files = collect_surql_files(&[schema_path.to_path_buf()]);
		let mut content = String::new();
		for file in &schema_files {
			match read_file(file) {
				Ok(c) => {
					content.push_str(&c);
					content.push('\n');
				}
				Err(e) => {
					eprintln!("{e}");
					return ExitCode::FAILURE;
				}
			}
		}
		Some(content)
	} else {
		None
	};

	#[cfg(feature = "test-runner")]
	return cmd_test_execute(&test_files, schema_content.as_deref());

	#[cfg(not(feature = "test-runner"))]
	cmd_test_syntax_only(&test_files, schema_content.as_deref())
}

#[cfg(not(feature = "test-runner"))]
fn cmd_test_syntax_only(test_files: &[PathBuf], schema_content: Option<&str>) -> ExitCode {
	eprintln!("Validating syntax for {} test file(s)...", test_files.len(),);
	eprintln!(
		"NOTE: Full test execution requires --features test-runner. \
		 Validating syntax only."
	);

	let mut passed = 0u32;
	let mut failed = 0u32;

	for file in test_files {
		let content = match read_file(file) {
			Ok(c) => c,
			Err(e) => {
				eprintln!("FAIL {}: {e}", file.display());
				failed += 1;
				continue;
			}
		};

		let full_content = if let Some(schema) = schema_content {
			format!("{schema}\n{content}")
		} else {
			content
		};

		match surql_parser::parse(&full_content) {
			Ok(_) => {
				eprintln!("SYNTAX OK {}", file.display());
				passed += 1;
			}
			Err(e) => {
				eprintln!("FAIL {}: {e}", file.display());
				failed += 1;
			}
		}
	}

	eprintln!();
	eprintln!(
		"{passed} passed, {failed} failed ({} total)",
		test_files.len()
	);

	if failed > 0 {
		ExitCode::FAILURE
	} else {
		ExitCode::SUCCESS
	}
}

#[cfg(feature = "test-runner")]
async fn run_test_file(schema_content: Option<&str>, test_content: &str) -> Result<(), String> {
	use surrealdb::{Surreal, engine::local::Mem};

	let db = Surreal::new::<Mem>(()).await.map_err(|e| e.to_string())?;
	db.use_ns("test")
		.use_db("test")
		.await
		.map_err(|e| e.to_string())?;

	if let Some(schema) = schema_content {
		db.query(schema)
			.await
			.map_err(|e| e.to_string())?
			.check()
			.map_err(|e| e.to_string())?;
	}

	db.query(test_content)
		.await
		.map_err(|e| e.to_string())?
		.check()
		.map_err(|e| e.to_string())?;

	Ok(())
}

#[cfg(feature = "test-runner")]
fn cmd_test_execute(test_files: &[PathBuf], schema_content: Option<&str>) -> ExitCode {
	eprintln!(
		"Running {} test file(s) against in-memory SurrealDB...",
		test_files.len(),
	);

	let rt = tokio::runtime::Runtime::new().unwrap_or_else(|e| {
		eprintln!("Failed to create tokio runtime: {e}");
		std::process::exit(1);
	});

	let mut passed = 0u32;
	let mut failed = 0u32;

	for file in test_files {
		let content = match read_file(file) {
			Ok(c) => c,
			Err(e) => {
				eprintln!("FAIL {}: {e}", file.display());
				failed += 1;
				continue;
			}
		};

		match rt.block_on(run_test_file(schema_content, &content)) {
			Ok(()) => {
				eprintln!("PASS {}", file.display());
				passed += 1;
			}
			Err(e) => {
				eprintln!("FAIL {}: {e}", file.display());
				failed += 1;
			}
		}
	}

	eprintln!();
	eprintln!(
		"{passed} passed, {failed} failed ({} total)",
		test_files.len()
	);

	if failed > 0 {
		ExitCode::FAILURE
	} else {
		ExitCode::SUCCESS
	}
}
