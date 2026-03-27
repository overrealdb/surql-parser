use zed_extension_api::{
	self as zed, ContextServerId, LanguageServerId, Project, Result, SlashCommand,
	SlashCommandArgumentCompletion, SlashCommandOutput, SlashCommandOutputSection, Worktree,
};

struct SurrealQlExtension;

impl zed::Extension for SurrealQlExtension {
	fn new() -> Self {
		Self
	}

	fn language_server_command(
		&mut self,
		_language_server_id: &LanguageServerId,
		worktree: &Worktree,
	) -> Result<zed::Command> {
		let path = worktree.which("surql-lsp").ok_or_else(|| {
			"surql-lsp not found in PATH. Install: cargo install --path lsp --features embedded-db"
				.to_string()
		})?;
		Ok(zed::Command {
			command: path,
			args: vec![],
			env: worktree.shell_env(),
		})
	}

	fn context_server_command(
		&mut self,
		_context_server_id: &ContextServerId,
		_project: &Project,
	) -> Result<zed::Command> {
		Ok(zed::Command {
			command: "surql-mcp".to_string(),
			args: vec![],
			env: vec![],
		})
	}

	fn run_slash_command(
		&self,
		command: SlashCommand,
		args: Vec<String>,
		_worktree: Option<&Worktree>,
	) -> Result<SlashCommandOutput, String> {
		match command.name.as_str() {
			"surql-schema" => run_schema_command(&args),
			"surql-relations" => read_cached_file("relations.md", "SurrealQL Relations"),
			"surql-info" => read_cached_file("info.md", "SurrealQL Info"),
			_ => Err(format!("Unknown command: {}", command.name)),
		}
	}

	fn complete_slash_command_argument(
		&self,
		command: SlashCommand,
		_args: Vec<String>,
	) -> Result<Vec<SlashCommandArgumentCompletion>, String> {
		if command.name == "surql-schema" {
			if let Ok(schema) = std::fs::read_to_string("schema.md") {
				let tables: Vec<SlashCommandArgumentCompletion> = schema
					.lines()
					.filter(|l| l.starts_with("## "))
					.filter_map(|l| {
						let path = l.trim_start_matches("## ").trim();
						Some(SlashCommandArgumentCompletion {
							label: path.to_string(),
							new_text: path.to_string(),
							run_command: true,
						})
					})
					.collect();
				return Ok(tables);
			}
		}
		Ok(vec![])
	}
}

fn read_cached_file(filename: &str, label: &str) -> Result<SlashCommandOutput, String> {
	let text = std::fs::read_to_string(filename).unwrap_or_else(|_| {
		"Not available yet. Save a .surql file to trigger LSP scan.".to_string()
	});
	let len = text.len();
	Ok(SlashCommandOutput {
		text,
		sections: vec![SlashCommandOutputSection {
			range: (0..len).into(),
			label: label.to_string(),
		}],
	})
}

fn run_schema_command(args: &[String]) -> Result<SlashCommandOutput, String> {
	let schema = std::fs::read_to_string("schema.md").unwrap_or_else(|_| {
		"Not available yet. Save a .surql file to trigger LSP scan.".to_string()
	});

	let filter = args.first().map(|s| s.to_lowercase());

	let text = if let Some(ref filter) = filter {
		filter_schema(&schema, filter)
	} else {
		schema
	};

	let len = text.len();
	Ok(SlashCommandOutput {
		text,
		sections: vec![SlashCommandOutputSection {
			range: (0..len).into(),
			label: if let Some(ref f) = filter {
				format!("SurrealQL Schema: {f}")
			} else {
				"SurrealQL Schema".to_string()
			},
		}],
	})
}

fn filter_schema(schema: &str, table_filter: &str) -> String {
	let mut result = String::new();
	let mut in_matching_section = false;

	for line in schema.lines() {
		if line.starts_with("## ") {
			in_matching_section = false;
		}

		if line.starts_with("## ") || line.starts_with("*") {
			let lower = line.to_lowercase();
			if lower.contains(table_filter) {
				in_matching_section = true;
			}
		}

		if in_matching_section {
			result.push_str(line);
			result.push('\n');
		} else if line.starts_with("```surql") || line.starts_with("```") {
			if in_matching_section {
				result.push_str(line);
				result.push('\n');
			}
		}

		// Also include lines that mention the table in DEFINE statements
		if !in_matching_section {
			let lower = line.to_lowercase();
			if (lower.contains(&format!("on {table_filter}"))
				|| lower.contains(&format!("record<{table_filter}>"))
				|| lower.contains(&format!("fn::{table_filter}")))
				&& !line.starts_with("*")
			{
				if result.is_empty() || !result.ends_with("```surql\n") {
					result.push_str("```surql\n");
				}
				result.push_str(line);
				result.push('\n');
			}
		}
	}

	if result.is_empty() {
		format!("No definitions found matching '{table_filter}'")
	} else {
		result
	}
}

zed::register_extension!(SurrealQlExtension);
