//! surql — CLI tool for SurrealQL files.
//!
//! Install: cargo install surql-parser --features cli
//!
//! Usage:
//!   surql check schema/**/*.surql     — validate files parse correctly
//!   surql schema schema/              — extract all definitions
//!   surql fmt file.surql              — format SurrealQL
//!   surql functions schema/           — list all fn::* definitions
//!   surql tables schema/              — list all table definitions

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
		/// Files or directories to check
		#[arg(required = true)]
		paths: Vec<PathBuf>,
	},
	/// Extract and display schema definitions
	Schema {
		/// Files or directories to analyze
		#[arg(required = true)]
		paths: Vec<PathBuf>,
	},
	/// Format SurrealQL files
	Fmt {
		/// Files to format
		#[arg(required = true)]
		paths: Vec<PathBuf>,
		/// Write formatted output back to file (default: stdout)
		#[arg(short, long)]
		write: bool,
	},
	/// List all function definitions
	Functions {
		/// Files or directories to analyze
		#[arg(required = true)]
		paths: Vec<PathBuf>,
	},
	/// List all table definitions
	Tables {
		/// Files or directories to analyze
		#[arg(required = true)]
		paths: Vec<PathBuf>,
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
		}
	}
	files.sort();
	files
}

fn read_surql(path: &PathBuf) -> String {
	let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
		eprintln!("Error reading {}: {e}", path.display());
		std::process::exit(1);
	});
	// Strip test header comments (/** ... */) if present
	if let Some(pos) = content.find("*/") {
		content[pos + 2..].trim().to_string()
	} else {
		content
	}
}

fn main() -> ExitCode {
	let cli = Cli::parse();

	match cli.command {
		Command::Check { paths } => cmd_check(&paths),
		Command::Schema { paths } => cmd_schema(&paths),
		Command::Fmt { paths, write } => cmd_fmt(&paths, write),
		Command::Functions { paths } => cmd_functions(&paths),
		Command::Tables { paths } => cmd_tables(&paths),
	}
}

fn cmd_check(paths: &[PathBuf]) -> ExitCode {
	let files = collect_surql_files(paths);
	if files.is_empty() {
		eprintln!("No .surql files found.");
		return ExitCode::FAILURE;
	}

	let mut passed = 0;
	let mut failed = 0;

	for file in &files {
		let content = read_surql(file);
		match surql_parser::parse(&content) {
			Ok(_) => {
				passed += 1;
			}
			Err(e) => {
				eprintln!("FAIL  {} — {e}", file.display());
				failed += 1;
			}
		}
	}

	eprintln!("{passed} passed, {failed} failed ({} files)", files.len());
	if failed > 0 {
		ExitCode::FAILURE
	} else {
		ExitCode::SUCCESS
	}
}

fn cmd_schema(paths: &[PathBuf]) -> ExitCode {
	let files = collect_surql_files(paths);
	let mut all = surql_parser::SchemaDefinitions::default();

	for file in &files {
		let content = read_surql(file);
		match surql_parser::extract_definitions(&content) {
			Ok(defs) => {
				all.tables.extend(defs.tables);
				all.fields.extend(defs.fields);
				all.indexes.extend(defs.indexes);
				all.functions.extend(defs.functions);
				all.analyzers.extend(defs.analyzers);
				all.events.extend(defs.events);
				all.params.extend(defs.params);
				all.namespaces.extend(defs.namespaces);
				all.databases.extend(defs.databases);
				all.users.extend(defs.users);
				all.accesses.extend(defs.accesses);
			}
			Err(e) => {
				eprintln!("Error parsing {}: {e}", file.display());
			}
		}
	}

	use surrealdb_types::{SqlFormat, ToSql};

	if !all.tables.is_empty() {
		println!("# Tables ({})", all.tables.len());
		for t in &all.tables {
			let mut s = String::new();
			t.fmt_sql(&mut s, SqlFormat::SingleLine);
			println!("  {s};");
		}
		println!();
	}

	if !all.fields.is_empty() {
		println!("# Fields ({})", all.fields.len());
		for f in &all.fields {
			let mut s = String::new();
			f.fmt_sql(&mut s, SqlFormat::SingleLine);
			println!("  {s};");
		}
		println!();
	}

	if !all.indexes.is_empty() {
		println!("# Indexes ({})", all.indexes.len());
		for i in &all.indexes {
			let mut s = String::new();
			i.fmt_sql(&mut s, SqlFormat::SingleLine);
			println!("  {s};");
		}
		println!();
	}

	if !all.functions.is_empty() {
		println!("# Functions ({})", all.functions.len());
		for f in &all.functions {
			let mut s = String::new();
			f.name.fmt_sql(&mut s, SqlFormat::SingleLine);
			println!("  fn::{s}");
		}
		println!();
	}

	if !all.analyzers.is_empty() {
		println!("# Analyzers ({})", all.analyzers.len());
		for a in &all.analyzers {
			let mut s = String::new();
			a.fmt_sql(&mut s, SqlFormat::SingleLine);
			println!("  {s};");
		}
		println!();
	}

	if !all.events.is_empty() {
		println!("# Events ({})", all.events.len());
		for e in &all.events {
			let mut s = String::new();
			e.fmt_sql(&mut s, SqlFormat::SingleLine);
			println!("  {s};");
		}
		println!();
	}

	ExitCode::SUCCESS
}

fn cmd_fmt(paths: &[PathBuf], write: bool) -> ExitCode {
	let files = collect_surql_files(paths);
	for file in &files {
		let content = read_surql(file);
		match surql_parser::parse(&content) {
			Ok(ast) => {
				let formatted = surql_parser::format(&ast);
				if write {
					std::fs::write(file, &formatted).unwrap_or_else(|e| {
						eprintln!("Error writing {}: {e}", file.display());
					});
					eprintln!("Formatted: {}", file.display());
				} else {
					println!("{formatted}");
				}
			}
			Err(e) => {
				eprintln!("Error parsing {}: {e}", file.display());
				return ExitCode::FAILURE;
			}
		}
	}
	ExitCode::SUCCESS
}

fn cmd_functions(paths: &[PathBuf]) -> ExitCode {
	let files = collect_surql_files(paths);
	for file in &files {
		let content = read_surql(file);
		match surql_parser::list_functions(&content) {
			Ok(fns) => {
				for name in fns {
					println!("fn::{name}  ({})", file.display());
				}
			}
			Err(e) => eprintln!("Error parsing {}: {e}", file.display()),
		}
	}
	ExitCode::SUCCESS
}

fn cmd_tables(paths: &[PathBuf]) -> ExitCode {
	let files = collect_surql_files(paths);
	for file in &files {
		let content = read_surql(file);
		match surql_parser::list_tables(&content) {
			Ok(tables) => {
				for name in tables {
					println!("{name}  ({})", file.display());
				}
			}
			Err(e) => eprintln!("Error parsing {}: {e}", file.display()),
		}
	}
	ExitCode::SUCCESS
}
