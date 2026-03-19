//! LSP Backend — implements the Language Server Protocol for SurrealQL.

use std::path::PathBuf;
use std::sync::RwLock;

use surql_parser::SchemaGraph;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::completion;
use crate::diagnostics;
use crate::document::DocumentStore;
use crate::formatting;

pub struct Backend {
	client: Client,
	documents: DocumentStore,
	schema: RwLock<SchemaGraph>,
	workspace_root: RwLock<Option<PathBuf>>,
}

impl Backend {
	pub fn new(client: Client) -> Self {
		Self {
			client,
			documents: DocumentStore::new(),
			schema: RwLock::new(SchemaGraph::default()),
			workspace_root: RwLock::new(None),
		}
	}

	/// Rebuild the workspace schema graph from all .surql files.
	fn rebuild_schema(&self) {
		let root = self.workspace_root.read().ok().and_then(|r| r.clone());
		if let Some(root) = root {
			match SchemaGraph::from_files(&root) {
				Ok(sg) => {
					if let Ok(mut schema) = self.schema.write() {
						*schema = sg;
					}
					tracing::info!("Schema graph rebuilt from {}", root.display());
				}
				Err(e) => {
					tracing::warn!("Failed to rebuild schema: {e}");
				}
			}
		}
	}

	/// Publish diagnostics for a document.
	async fn publish_diagnostics(&self, uri: Url) {
		if let Some(source) = self.documents.get(&uri) {
			let diags = diagnostics::compute(&source);
			self.client.publish_diagnostics(uri, diags, None).await;
		}
	}
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
	async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
		// Store workspace root
		if let Some(root) = params.root_uri {
			if let Ok(path) = root.to_file_path() {
				if let Ok(mut wr) = self.workspace_root.write() {
					*wr = Some(path);
				}
			}
		}

		Ok(InitializeResult {
			capabilities: ServerCapabilities {
				text_document_sync: Some(TextDocumentSyncCapability::Kind(
					TextDocumentSyncKind::FULL,
				)),
				completion_provider: Some(CompletionOptions {
					trigger_characters: Some(vec![".".into(), ":".into(), "$".into()]),
					..Default::default()
				}),
				document_formatting_provider: Some(OneOf::Left(true)),
				hover_provider: Some(HoverProviderCapability::Simple(true)),
				definition_provider: Some(OneOf::Left(true)),
				..Default::default()
			},
			..Default::default()
		})
	}

	async fn initialized(&self, _: InitializedParams) {
		tracing::info!("SurrealQL LSP initialized");
		self.rebuild_schema();
	}

	async fn shutdown(&self) -> Result<()> {
		Ok(())
	}

	async fn did_open(&self, params: DidOpenTextDocumentParams) {
		let uri = params.text_document.uri.clone();
		self.documents.open(uri.clone(), params.text_document.text);
		self.publish_diagnostics(uri).await;
	}

	async fn did_change(&self, params: DidChangeTextDocumentParams) {
		let uri = params.text_document.uri.clone();
		if let Some(change) = params.content_changes.into_iter().last() {
			self.documents.update(&uri, change.text);
		}
		self.publish_diagnostics(uri).await;
	}

	async fn did_save(&self, params: DidSaveTextDocumentParams) {
		// Rebuild schema on save (any .surql file might have changed)
		self.rebuild_schema();
		self.publish_diagnostics(params.text_document.uri).await;
	}

	async fn did_close(&self, params: DidCloseTextDocumentParams) {
		self.documents.close(&params.text_document.uri);
		// Clear diagnostics for closed document
		self.client
			.publish_diagnostics(params.text_document.uri, vec![], None)
			.await;
	}

	async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
		let uri = &params.text_document.uri;
		let source = match self.documents.get(uri) {
			Some(s) => s,
			None => return Ok(None),
		};
		Ok(formatting::format_document(&source))
	}

	async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
		let uri = &params.text_document_position.text_document.uri;
		let position = params.text_document_position.position;
		let source = match self.documents.get(uri) {
			Some(s) => s,
			None => return Ok(None),
		};
		let schema = self.schema.read().ok();
		let items = completion::complete(&source, position, schema.as_deref());
		Ok(Some(CompletionResponse::Array(items)))
	}

	async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
		let uri = &params.text_document_position_params.text_document.uri;
		let position = params.text_document_position_params.position;
		let source = match self.documents.get(uri) {
			Some(s) => s,
			None => return Ok(None),
		};
		let schema = self.schema.read().ok();
		let schema = schema.as_deref();

		// Find the word at cursor
		let word = word_at_position(&source, position);
		if word.is_empty() {
			return Ok(None);
		}

		// Look up in schema
		if let Some(sg) = schema {
			// Table hover
			if let Some(table) = sg.table(&word) {
				let schema_type = if table.full {
					"SCHEMAFULL"
				} else {
					"SCHEMALESS"
				};
				let fields = sg.fields_of(&word);
				let field_list = fields
					.iter()
					.map(|f| {
						let kind = f.kind.as_deref().unwrap_or("any");
						format!("  {} : {}", f.name, kind)
					})
					.collect::<Vec<_>>()
					.join("\n");
				let content = format!(
					"**TABLE** `{word}` ({schema_type})\n\n**Fields:**\n```\n{field_list}\n```"
				);
				return Ok(Some(Hover {
					contents: HoverContents::Markup(MarkupContent {
						kind: MarkupKind::Markdown,
						value: content,
					}),
					range: None,
				}));
			}

			// Function hover (strip fn:: prefix if present)
			let fn_name = word.strip_prefix("fn::").unwrap_or(&word);
			if let Some(func) = sg.function(fn_name) {
				let args = func
					.args
					.iter()
					.map(|(n, t)| format!("{n}: {t}"))
					.collect::<Vec<_>>()
					.join(", ");
				let ret = func
					.returns
					.as_ref()
					.map(|r| format!(" -> {r}"))
					.unwrap_or_default();
				let content = format!(
					"**FUNCTION** `fn::{}`\n\n```\nfn::{}({args}){ret}\n```",
					func.name, func.name
				);
				return Ok(Some(Hover {
					contents: HoverContents::Markup(MarkupContent {
						kind: MarkupKind::Markdown,
						value: content,
					}),
					range: None,
				}));
			}
		}

		Ok(None)
	}

	async fn goto_definition(
		&self,
		params: GotoDefinitionParams,
	) -> Result<Option<GotoDefinitionResponse>> {
		// TODO: implement using SourceLocation from SchemaGraph
		let _ = params;
		Ok(None)
	}
}

/// Extract the word (identifier) at a cursor position.
pub(crate) fn word_at_position(source: &str, position: Position) -> String {
	let line = match source.lines().nth(position.line as usize) {
		Some(l) => l,
		None => return String::new(),
	};
	let col = position.character as usize;
	let bytes = line.as_bytes();

	// Find word boundaries
	let start = (0..col)
		.rev()
		.take_while(|&i| {
			i < bytes.len()
				&& (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b':')
		})
		.last()
		.unwrap_or(col);

	let end = (col..bytes.len())
		.take_while(|&i| bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b':')
		.last()
		.map(|i| i + 1)
		.unwrap_or(col);

	if start <= end && end <= line.len() {
		line[start..end].to_string()
	} else {
		String::new()
	}
}
