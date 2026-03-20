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
		_args: Vec<String>,
		_worktree: Option<&Worktree>,
	) -> Result<SlashCommandOutput, String> {
		match command.name.as_str() {
			"surql-schema" => {
				// LSP writes schema.md to the extension work dir (WASM cwd = ".")
				let text = std::fs::read_to_string("schema.md").unwrap_or_else(|_| {
					"Schema not available. Open a .surql file and save it to trigger LSP scan."
						.to_string()
				});
				let len = text.len();
				Ok(SlashCommandOutput {
					text,
					sections: vec![SlashCommandOutputSection {
						range: (0..len).into(),
						label: "SurrealQL Schema".to_string(),
					}],
				})
			}
			_ => Err(format!("Unknown command: {}", command.name)),
		}
	}

	fn complete_slash_command_argument(
		&self,
		_command: SlashCommand,
		_args: Vec<String>,
	) -> Result<Vec<SlashCommandArgumentCompletion>, String> {
		Ok(vec![])
	}
}

zed::register_extension!(SurrealQlExtension);
