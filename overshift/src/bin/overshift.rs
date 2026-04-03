use clap::{Parser, Subcommand};
use surrealdb::engine::any;

/// overshift — shared migration engine for the overrealdb ecosystem.
#[derive(Parser)]
#[command(name = "overshift", version, about)]
struct Cli {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
enum Command {
	/// Show what will be done (dry-run).
	Plan {
		/// Path to the surql/ project directory.
		path: String,
		/// SurrealDB connection URL (default: ws://localhost:8000).
		#[arg(long, default_value = "ws://localhost:8000")]
		url: String,
	},
	/// Apply pending migrations and declarative schema.
	Apply {
		/// Path to the surql/ project directory.
		path: String,
		/// SurrealDB connection URL.
		#[arg(long, default_value = "ws://localhost:8000")]
		url: String,
		/// Dry-run mode (same as `plan`).
		#[arg(long)]
		dry_run: bool,
		/// Skip schema module application (use when schema is already applied).
		#[arg(long, env = "OVERSHIFT_SKIP_SCHEMA")]
		skip_schema: bool,
		/// Only verify schema matches between shadow DB and target (no apply).
		#[cfg(feature = "shadow")]
		#[arg(long)]
		verify_only: bool,
	},
	/// Generate `generated/current.surql` schema snapshot.
	Snapshot {
		/// Path to the surql/ project directory.
		path: String,
		/// Check mode: fail if snapshot is outdated (for CI).
		#[arg(long)]
		check: bool,
	},
	/// Validate that all schema functions exist in the database.
	Validate {
		/// Path to the surql/ project directory.
		path: String,
		/// SurrealDB connection URL.
		#[arg(long, default_value = "ws://localhost:8000")]
		url: String,
	},
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt::init();
	let cli = Cli::parse();

	match cli.command {
		Command::Plan { path, url } => {
			let manifest = overshift::Manifest::load(&path)?;
			let db = any::connect(&url).await?;
			let plan = overshift::plan(&db, &manifest).await?;
			plan.print();
		}
		Command::Apply {
			path,
			url,
			dry_run,
			skip_schema,
			#[cfg(feature = "shadow")]
			verify_only,
		} => {
			let manifest = overshift::Manifest::load(&path)?;

			#[cfg(feature = "shadow")]
			if verify_only {
				let shadow = overshift::shadow::apply_to_shadow(&manifest).await?;
				if !shadow.errors.is_empty() {
					for err in &shadow.errors {
						eprintln!("shadow error: {err}");
					}
				}

				let db = any::connect(&url).await?;
				db.use_ns(&manifest.meta.ns)
					.use_db(&manifest.meta.db)
					.await?;
				let target_info = overshift::validate::query_db_info(&db).await?;

				let diff = overshift::validate::compare_db_info(&shadow.db_info, &target_info);

				if diff.is_empty() {
					println!("Schema matches: shadow DB and target are in sync.");
					std::process::exit(0);
				} else {
					println!("Schema drift detected:");
					print!("{diff}");
					std::process::exit(1);
				}
			}

			let db = any::connect(&url).await?;
			let mut plan = overshift::plan(&db, &manifest).await?;

			if skip_schema && !plan.schema_modules.is_empty() {
				println!(
					"Skipping {} schema modules (--skip-schema)",
					plan.schema_modules.len()
				);
				plan.schema_modules.clear();
			}

			if dry_run {
				plan.print();
			} else {
				plan.print();
				println!();
				let result = plan.apply(&db).await?;
				println!(
					"Done: {} migrations applied, {} schema modules applied (instance {})",
					result.applied_migrations, result.applied_modules, result.instance_id,
				);
			}
		}
		Command::Snapshot { path, check } => {
			let manifest = overshift::Manifest::load(&path)?;

			if check {
				if overshift::snapshot::check(&manifest)? {
					println!("Snapshot is up to date.");
				} else {
					eprintln!(
						"Snapshot is outdated. Run `overshift snapshot {}` to update.",
						path,
					);
					std::process::exit(1);
				}
			} else {
				overshift::snapshot::write(&manifest)?;
				println!(
					"Snapshot written to {}",
					manifest.generated_dir()?.join("current.surql").display(),
				);
			}
		}
		Command::Validate { path, url } => {
			let manifest = overshift::Manifest::load(&path)?;
			let db = any::connect(&url).await?;

			let modules = overshift::schema::load_schema_modules(&manifest)?;
			let functions = overshift::schema::extract_function_names(&modules)?;

			if functions.is_empty() {
				println!("No functions to validate.");
			} else {
				db.use_ns(&manifest.meta.ns)
					.use_db(&manifest.meta.db)
					.await?;
				overshift::validate::validate_functions(&db, &functions).await?;
				println!("All {} functions validated.", functions.len());
			}
		}
	}

	Ok(())
}
