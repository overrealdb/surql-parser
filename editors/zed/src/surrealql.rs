use zed_extension_api::{self as zed, LanguageServerId, Result};

struct SurrealQlExtension;

impl zed::Extension for SurrealQlExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // Look for surql-lsp in PATH (installed via `cargo install surql-lsp`)
        let path = worktree
            .which("surql-lsp")
            .ok_or_else(|| {
                "surql-lsp not found in PATH. Install it with: cargo install --path lsp (from surql-parser repo)".to_string()
            })?;

        Ok(zed::Command {
            command: path,
            args: vec![],
            env: worktree.shell_env(),
        })
    }
}

zed::register_extension!(SurrealQlExtension);
