//! LSP Backend — implements the Language Server Protocol for SurrealQL.

use std::path::PathBuf;
use std::sync::RwLock;

use dashmap::DashMap;
use surql_parser::SchemaGraph;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::completion;
use crate::completion::position_to_byte_offset;
use crate::context;
use crate::diagnostics;
use crate::document::DocumentStore;
use crate::embedded;
use crate::formatting;
use crate::signature;

/// Semantic token types for SurrealQL highlighting in embedded contexts.
const SEMANTIC_TOKEN_TYPES: &[SemanticTokenType] = &[
	SemanticTokenType::KEYWORD,  // 0 — SELECT, FROM, DEFINE, etc.
	SemanticTokenType::FUNCTION, // 1 — string::len, fn::greet
	SemanticTokenType::VARIABLE, // 2 — $param
	SemanticTokenType::STRING,   // 3 — 'literal'
	SemanticTokenType::NUMBER,   // 4 — 42, 3.14
	SemanticTokenType::OPERATOR, // 5 — =, >, AND, OR
	SemanticTokenType::TYPE,     // 6 — table names in type positions
	SemanticTokenType::COMMENT,  // 7 — -- comment
];

pub struct Backend {
	client: Client,
	documents: DocumentStore,
	schema: RwLock<SchemaGraph>,
	document_schemas: DashMap<Url, SchemaGraph>,
	workspace_root: RwLock<Option<PathBuf>>,
	format_enabled: bool,
	#[cfg(feature = "embedded-db")]
	embedded: tokio::sync::RwLock<Option<crate::embedded_db::DualEngine>>,
}

impl Backend {
	pub fn new(client: Client) -> Self {
		Self {
			client,
			documents: DocumentStore::new(),
			schema: RwLock::new(SchemaGraph::default()),
			document_schemas: DashMap::new(),
			workspace_root: RwLock::new(None),
			format_enabled: cfg!(feature = "canonical-format"),
			#[cfg(feature = "embedded-db")]
			embedded: tokio::sync::RwLock::new(None),
		}
	}

	/// Rebuild the workspace schema graph from all .surql files.
	/// Also writes `surql-lsp-out/files.json` manifest for the Zed extension.
	fn rebuild_schema(&self) {
		let root = match self.workspace_root.read() {
			Ok(r) => r.clone(),
			Err(e) => {
				tracing::error!("workspace_root lock poisoned: {e}");
				return;
			}
		};
		if let Some(root) = root {
			match SchemaGraph::from_files(&root) {
				Ok(sg) => match self.schema.write() {
					Ok(mut schema) => {
						tracing::info!(
							"Schema rebuilt: {} tables, {} functions from {}",
							sg.table_names().count(),
							sg.function_names().count(),
							root.display()
						);
						*schema = sg;
					}
					Err(e) => tracing::error!("schema lock poisoned: {e}"),
				},
				Err(e) => {
					tracing::warn!("Failed to rebuild schema: {e}");
				}
			}
			write_file_manifest(&root);
		} else {
			tracing::debug!("rebuild_schema: no workspace root set");
		}
	}

	/// Get the effective schema for a document: workspace schema + document overlay.
	fn effective_schema(&self, uri: &Url) -> SchemaGraph {
		let mut schema = match self.schema.read() {
			Ok(s) => s.clone(),
			Err(e) => {
				tracing::error!("schema lock poisoned in effective_schema: {e}");
				SchemaGraph::default()
			}
		};
		if let Some(doc_schema) = self.document_schemas.get(uri) {
			schema.merge(doc_schema.clone());
		}
		schema
	}

	/// Resolve a schema SourceLocation to an LSP GotoDefinitionResponse.
	/// Tries in-memory documents first, falls back to reading from disk.
	fn resolve_source_location(
		&self,
		loc: &surql_parser::schema_graph::SourceLocation,
	) -> Option<GotoDefinitionResponse> {
		let target_uri = Url::from_file_path(&loc.file).ok()?;
		// Prefer in-memory document content (works for unsaved files and tests)
		let content = self
			.documents
			.get(&target_uri)
			.or_else(|| std::fs::read_to_string(&loc.file).ok())?;
		let pos = byte_offset_to_position(&content, loc.offset);
		Some(GotoDefinitionResponse::Scalar(Location {
			uri: target_uri,
			range: Range {
				start: pos,
				end: pos,
			},
		}))
	}

	fn is_rust_file(uri: &Url) -> bool {
		uri.path().ends_with(".rs")
	}

	async fn publish_diagnostics(&self, uri: Url) {
		if let Some(source) = self.documents.get(&uri) {
			if Self::is_rust_file(&uri) {
				self.publish_rust_diagnostics(uri, &source).await;
				return;
			}

			let result = diagnostics::compute_with_recovery(&source);

			// Extract schema definitions from recovered AST for live completions
			match surql_parser::extract_definitions_from_ast(&result.statements) {
				Ok(defs) => {
					let mut graph = SchemaGraph::from_definitions(&defs);
					if let Ok(path) = uri.to_file_path() {
						graph.attach_source_locations(&source, &path);
					}
					self.document_schemas.insert(uri.clone(), graph);
				}
				Err(e) => {
					tracing::trace!("Could not extract definitions from recovered AST: {e}");
				}
			}

			#[allow(unused_mut)]
			let mut all_diagnostics = result.diagnostics;

			// Schema-aware: warn about undefined table references in DML
			let schema = self.effective_schema(&uri);
			if schema.table_names().count() > 0 {
				for table_ref in context::extract_table_references(&source) {
					if schema.table(&table_ref.name).is_none() {
						all_diagnostics.push(Diagnostic {
							range: Range {
								start: Position {
									line: table_ref.line,
									character: table_ref.col,
								},
								end: Position {
									line: table_ref.line,
									character: table_ref.col + table_ref.len,
								},
							},
							severity: Some(DiagnosticSeverity::WARNING),
							source: Some("surql-schema".into()),
							message: format!(
								"Table '{}' is not defined in workspace",
								table_ref.name
							),
							..Default::default()
						});
					}
				}
			}

			self.client
				.publish_diagnostics(uri, all_diagnostics, None)
				.await;
		}
	}

	async fn publish_rust_diagnostics(&self, uri: Url, source: &str) {
		let regions = embedded::extract_surql_from_rust(source);
		let mut diagnostics = Vec::new();

		for region in &regions {
			if region.kind != embedded::RegionKind::Statement {
				continue;
			}
			let content = region.content.clone();
			let parse_result =
				std::panic::catch_unwind(move || surql_parser::parse_for_diagnostics(&content));
			let diags = match parse_result {
				Ok(Err(diags)) => diags,
				_ => continue,
			};
			for d in diags {
				diagnostics.push(Diagnostic {
					range: Range {
						start: Position {
							line: region.line + (d.line.saturating_sub(1)) as u32,
							character: if d.line == 1 {
								region.col + (d.column.saturating_sub(1)) as u32
							} else {
								(d.column.saturating_sub(1)) as u32
							},
						},
						end: Position {
							line: region.line + (d.end_line.saturating_sub(1)) as u32,
							character: if d.end_line == 1 {
								region.col + (d.end_column.saturating_sub(1)) as u32
							} else {
								(d.end_column.saturating_sub(1)) as u32
							},
						},
					},
					severity: Some(DiagnosticSeverity::ERROR),
					source: Some("surql".into()),
					message: d.message,
					..Default::default()
				});
			}
		}

		self.client
			.publish_diagnostics(uri, diagnostics, None)
			.await;
	}
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
	async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
		// Store workspace root and write file manifest early (before initialized)
		// so the Zed extension can discover .surql files via slash commands
		if let Some(root) = params.root_uri
			&& let Ok(path) = root.to_file_path()
			&& let Ok(mut wr) = self.workspace_root.write()
		{
			write_file_manifest(&path);
			*wr = Some(path);
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
				references_provider: Some(OneOf::Left(true)),
				signature_help_provider: Some(SignatureHelpOptions {
					trigger_characters: Some(vec!["(".into(), ",".into()]),
					retrigger_characters: Some(vec![",".into()]),
					..Default::default()
				}),
				document_symbol_provider: Some(OneOf::Left(true)),
				code_lens_provider: Some(CodeLensOptions {
					resolve_provider: Some(false),
				}),
				semantic_tokens_provider: Some(
					SemanticTokensServerCapabilities::SemanticTokensOptions(
						SemanticTokensOptions {
							legend: SemanticTokensLegend {
								token_types: SEMANTIC_TOKEN_TYPES.to_vec(),
								token_modifiers: vec![],
							},
							full: Some(SemanticTokensFullOptions::Bool(true)),
							range: None,
							..Default::default()
						},
					),
				),
				..Default::default()
			},
			..Default::default()
		})
	}

	async fn initialized(&self, _: InitializedParams) {
		tracing::info!("SurrealQL LSP initialized");
		self.rebuild_schema();

		#[cfg(feature = "embedded-db")]
		{
			let workspace_path = self.workspace_root.read().ok().and_then(|r| r.clone());
			match crate::embedded_db::DualEngine::start().await {
				Ok(engine) => {
					if let Some(root) = workspace_path
						&& let Err(e) = engine.apply_migrations(&root).await
					{
						tracing::warn!("Failed to apply migrations: {e}");
					}
					*self.embedded.write().await = Some(engine);
				}
				Err(e) => tracing::error!("Failed to start embedded SurrealDB: {e}"),
			}
		}
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

		#[cfg(feature = "embedded-db")]
		{
			if let Some(ref engine) = *self.embedded.read().await {
				let workspace_path = self.workspace_root.read().ok().and_then(|r| r.clone());
				if let Some(root) = workspace_path
					&& let Err(e) = engine.apply_migrations(&root).await
				{
					tracing::warn!("Failed to reapply migrations on save: {e}");
				}
			}
		}

		self.publish_diagnostics(params.text_document.uri).await;
	}

	async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
		let uri = &params.text_document.uri;
		let source = match self.documents.get(uri) {
			Some(s) => s,
			None => return Ok(None),
		};
		Ok(formatting::format_document(&source, self.format_enabled))
	}

	async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
		let uri = &params.text_document_position.text_document.uri;
		let position = params.text_document_position.position;
		let source = match self.documents.get(uri) {
			Some(s) => s,
			None => return Ok(None),
		};
		let schema = self.effective_schema(uri);
		let items = completion::complete(&source, position, Some(&schema));
		Ok(Some(CompletionResponse::Array(items)))
	}

	async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
		let uri = &params.text_document_position_params.text_document.uri;
		let position = params.text_document_position_params.position;
		let source = match self.documents.get(uri) {
			Some(s) => s,
			None => return Ok(None),
		};

		// For Rust files, check if cursor is inside a surql macro string
		if Self::is_rust_file(uri) {
			let regions = embedded::extract_surql_from_rust(&source);
			let byte_off = position_to_byte_offset(&source, position);
			let schema = self.effective_schema(uri);
			for region in &regions {
				let region_end = region.offset + region.content.len();
				if byte_off >= region.offset && byte_off <= region_end {
					// For FunctionName regions, show function info
					if region.kind == embedded::RegionKind::FunctionName {
						let fn_name = region
							.content
							.strip_prefix("fn::")
							.unwrap_or(&region.content);
						if let Some(func) = schema.function(fn_name) {
							let content = format_function_hover(func);
							return Ok(Some(Hover {
								contents: HoverContents::Markup(MarkupContent {
									kind: MarkupKind::Markdown,
									value: content,
								}),
								range: None,
							}));
						}
						break;
					}

					// For Statement regions, provide keyword/builtin hover
					let local_off = byte_off - region.offset;
					let local_pos = byte_offset_to_position(&region.content, local_off);
					let word = word_at_position(&region.content, local_pos);
					if !word.is_empty() {
						if let Some(builtin) = surql_parser::builtin_function(&word) {
							let ns = builtin.name.split("::").next().unwrap_or(builtin.name);
							let content = format!(
								"**{}**\n\n{}\n\n```surql\n{}\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/functions/database/{ns})",
								builtin.name,
								builtin.description,
								builtin.signatures.join("\n"),
							);
							return Ok(Some(Hover {
								contents: HoverContents::Markup(MarkupContent {
									kind: MarkupKind::Markdown,
									value: content,
								}),
								range: None,
							}));
						}
						// Schema lookups for tables/functions in embedded SQL
						let fn_name = word.strip_prefix("fn::").unwrap_or(&word);
						if let Some(func) = schema.function(fn_name) {
							let content = format_function_hover(func);
							return Ok(Some(Hover {
								contents: HoverContents::Markup(MarkupContent {
									kind: MarkupKind::Markdown,
									value: content,
								}),
								range: None,
							}));
						}
						if let Some(table) = schema.table(&word) {
							let content = format_table_hover(&word, table, schema.fields_of(&word));
							return Ok(Some(Hover {
								contents: HoverContents::Markup(MarkupContent {
									kind: MarkupKind::Markdown,
									value: content,
								}),
								range: None,
							}));
						}
						if let Some(doc) = keyword_documentation(&word) {
							return Ok(Some(Hover {
								contents: HoverContents::Markup(MarkupContent {
									kind: MarkupKind::Markdown,
									value: doc.to_string(),
								}),
								range: None,
							}));
						}
					}
					break;
				}
			}
			return Ok(None);
		}

		let schema = self.effective_schema(uri);

		// Find the word at cursor
		let (word, word_range) = word_and_range_at_position(&source, position);
		tracing::debug!(
			"hover: word={word:?} at line={} col={}, tables={}, functions={}",
			position.line,
			position.character,
			schema.table_names().count(),
			schema.function_names().count(),
		);
		if word.is_empty() {
			return Ok(None);
		}

		// Table hover
		if let Some(table) = schema.table(&word) {
			let content = format_table_hover(&word, table, schema.fields_of(&word));
			return Ok(Some(Hover {
				contents: HoverContents::Markup(MarkupContent {
					kind: MarkupKind::Markdown,
					value: content,
				}),
				range: word_range,
			}));
		}

		// User-defined function hover (strip fn:: prefix if present)
		let fn_name = word.strip_prefix("fn::").unwrap_or(&word);
		if let Some(func) = schema.function(fn_name) {
			let content = format_function_hover(func);
			return Ok(Some(Hover {
				contents: HoverContents::Markup(MarkupContent {
					kind: MarkupKind::Markdown,
					value: content,
				}),
				range: word_range,
			}));
		}

		// Built-in function hover (string::len, array::add, etc.)
		if let Some(builtin) = surql_parser::builtin_function(&word) {
			let sigs = builtin
				.signatures
				.iter()
				.map(|s| s.to_string())
				.collect::<Vec<_>>()
				.join("\n");
			let ns = builtin.name.split("::").next().unwrap_or(builtin.name);
			let docs_url = format!("https://surrealdb.com/docs/surrealql/functions/database/{ns}");
			let content = format!(
				"**{}**\n\n{}\n\n```surql\n{}\n```\n\n[Docs]({docs_url})",
				builtin.name, builtin.description, sigs
			);
			return Ok(Some(Hover {
				contents: HoverContents::Markup(MarkupContent {
					kind: MarkupKind::Markdown,
					value: content,
				}),
				range: word_range,
			}));
		}

		// Field hover — context-aware: show only the field from the current table
		if let Some(table_name) = context::table_context_at_position(&source, position)
			&& let Some(f) = schema.field_on(&table_name, &word)
		{
			let kind = f.kind.as_deref().unwrap_or("any");
			let readonly = if f.readonly { " READONLY" } else { "" };
			let default = f
				.default
				.as_ref()
				.map(|d| format!(" DEFAULT {d}"))
				.unwrap_or_default();
			let comment = f
				.comment
				.as_ref()
				.map(|c| format!("  -- {c}"))
				.unwrap_or_default();
			let entry = format!(
				"  {table_name}.{} : {kind}{default}{readonly}{comment}",
				f.name
			);
			let content = format!("**FIELD** `{word}`\n\n```surql\n{entry}\n```");
			return Ok(Some(Hover {
				contents: HoverContents::Markup(MarkupContent {
					kind: MarkupKind::Markdown,
					value: content,
				}),
				range: word_range,
			}));
		}

		// Field hover fallback — show type and which table(s) it belongs to
		let fields = schema.find_field(&word);
		if !fields.is_empty() {
			let entries: Vec<String> = fields
				.iter()
				.map(|(table, f)| {
					let kind = f.kind.as_deref().unwrap_or("any");
					let readonly = if f.readonly { " READONLY" } else { "" };
					let default = f
						.default
						.as_ref()
						.map(|d| format!(" DEFAULT {d}"))
						.unwrap_or_default();
					let comment = f
						.comment
						.as_ref()
						.map(|c| format!("  -- {c}"))
						.unwrap_or_default();
					format!("  {table}.{} : {kind}{default}{readonly}{comment}", f.name)
				})
				.collect();
			let content = format!(
				"**FIELD** `{word}`\n\n```surql\n{}\n```",
				entries.join("\n")
			);
			return Ok(Some(Hover {
				contents: HoverContents::Markup(MarkupContent {
					kind: MarkupKind::Markdown,
					value: content,
				}),
				range: word_range,
			}));
		}

		// SurrealQL type documentation (option, record, array, etc.)
		if let Some(doc) = type_documentation(&word, &schema) {
			return Ok(Some(Hover {
				contents: HoverContents::Markup(MarkupContent {
					kind: MarkupKind::Markdown,
					value: doc,
				}),
				range: word_range,
			}));
		}

		// Record ID: split on ':' to resolve the table part (e.g., user:alice -> user)
		if let Some(table_name) = word.split(':').next()
			&& table_name != word
			&& let Some(table) = schema.table(table_name)
		{
			let content = format_table_hover(table_name, table, schema.fields_of(table_name));
			return Ok(Some(Hover {
				contents: HoverContents::Markup(MarkupContent {
					kind: MarkupKind::Markdown,
					value: content,
				}),
				range: word_range,
			}));
		}

		// SurrealQL keyword documentation
		if let Some(doc) = keyword_documentation(&word) {
			return Ok(Some(Hover {
				contents: HoverContents::Markup(MarkupContent {
					kind: MarkupKind::Markdown,
					value: doc.to_string(),
				}),
				range: word_range,
			}));
		}

		Ok(None)
	}

	async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
		let uri = &params.text_document_position_params.text_document.uri;
		let position = params.text_document_position_params.position;
		let source = match self.documents.get(uri) {
			Some(s) => s,
			None => return Ok(None),
		};
		let schema = self.effective_schema(uri);
		Ok(signature::signature_help(&source, position, Some(&schema)))
	}

	async fn goto_definition(
		&self,
		params: GotoDefinitionParams,
	) -> Result<Option<GotoDefinitionResponse>> {
		let uri = &params.text_document_position_params.text_document.uri;
		let position = params.text_document_position_params.position;
		let source = match self.documents.get(uri) {
			Some(s) => s,
			None => return Ok(None),
		};

		let word = word_at_position(&source, position);
		if word.is_empty() {
			return Ok(None);
		}

		let schema = self.effective_schema(uri);

		// Try to find definition location
		let fn_name = word.strip_prefix("fn::").unwrap_or(&word);
		if let Some(func) = schema.function(fn_name)
			&& let Some(ref loc) = func.source
			&& let Some(resp) = self.resolve_source_location(loc)
		{
			return Ok(Some(resp));
		}

		if let Some(table) = schema.table(&word)
			&& let Some(ref loc) = table.source
			&& let Some(resp) = self.resolve_source_location(loc)
		{
			return Ok(Some(resp));
		}

		// Try field lookup — find first field with this name across all tables
		let fields = schema.find_field(&word);
		if let Some((_, field)) = fields.first()
			&& let Some(ref loc) = field.source
			&& let Some(resp) = self.resolve_source_location(loc)
		{
			return Ok(Some(resp));
		}

		Ok(None)
	}

	async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
		let uri = &params.text_document_position.text_document.uri;
		let position = params.text_document_position.position;
		let source = match self.documents.get(uri) {
			Some(s) => s,
			None => return Ok(None),
		};

		let word = word_at_position(&source, position);
		if word.is_empty() {
			return Ok(None);
		}

		// Find all occurrences of the word in all open documents
		let mut locations = Vec::new();
		for entry in self.documents.all() {
			let (doc_uri, doc_source) = entry;
			find_word_occurrences(&doc_source, &word, &doc_uri, &mut locations);
		}

		if locations.is_empty() {
			Ok(None)
		} else {
			Ok(Some(locations))
		}
	}

	async fn semantic_tokens_full(
		&self,
		params: SemanticTokensParams,
	) -> Result<Option<SemanticTokensResult>> {
		let uri = &params.text_document.uri;
		let source = match self.documents.get(uri) {
			Some(s) => s,
			None => return Ok(None),
		};

		// Only provide semantic tokens for Rust files (embedded SurrealQL)
		if !Self::is_rust_file(uri) {
			return Ok(None);
		}

		let regions = embedded::extract_surql_from_rust(&source);
		if regions.is_empty() {
			return Ok(None);
		}

		let mut tokens = Vec::new();
		for region in &regions {
			if region.kind != embedded::RegionKind::Statement {
				continue;
			}
			tokenize_surql_region(&source, region, &mut tokens);
		}

		if tokens.is_empty() {
			return Ok(None);
		}

		Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
			result_id: None,
			data: tokens,
		})))
	}

	async fn document_symbol(
		&self,
		params: DocumentSymbolParams,
	) -> Result<Option<DocumentSymbolResponse>> {
		let uri = &params.text_document.uri;
		if Self::is_rust_file(uri) {
			return Ok(None);
		}
		let source = match self.documents.get(uri) {
			Some(s) => s,
			None => return Ok(None),
		};
		// Use document-only schema (not workspace) so ranges match current file
		let schema = self
			.document_schemas
			.get(uri)
			.map(|s| s.clone())
			.unwrap_or_default();
		let src = source.clone();
		match std::panic::catch_unwind(move || build_document_symbols(&src, &schema)) {
			Ok(result) => Ok(result),
			Err(_) => {
				tracing::error!("panic in build_document_symbols for {}", uri.path());
				Ok(None)
			}
		}
	}

	async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
		let uri = &params.text_document.uri;
		if Self::is_rust_file(uri) {
			return Ok(None);
		}
		let source = match self.documents.get(uri) {
			Some(s) => s,
			None => return Ok(None),
		};
		// Use document-only schema so code lens positions match current file
		let schema = self
			.document_schemas
			.get(uri)
			.map(|s| s.clone())
			.unwrap_or_default();
		let src = source.clone();
		match std::panic::catch_unwind(move || build_code_lenses(&src, &schema)) {
			Ok(result) => Ok(result),
			Err(_) => {
				tracing::error!("panic in build_code_lenses for {}", uri.path());
				Ok(None)
			}
		}
	}

	async fn did_close(&self, params: DidCloseTextDocumentParams) {
		let uri = params.text_document.uri;
		self.documents.close(&uri);
		self.document_schemas.remove(&uri);
		self.client.publish_diagnostics(uri, vec![], None).await;
	}
}

/// Write file manifest and schema cache for the Zed extension.
///
/// Writes to two locations:
/// 1. `surql-lsp-out/files.json` in project root (for general use)
/// 2. Zed extension work dir (WASM can read via `std::fs` from `.`)
fn write_file_manifest(root: &std::path::Path) {
	let mut files = Vec::new();
	collect_manifest_files(root, root, &mut files);
	files.sort();

	// Build schema cache content
	let mut schema_text = String::new();
	let mut file_count = 0;
	for rel_path in &files {
		let full_path = root.join(rel_path);
		let content = match std::fs::read_to_string(&full_path) {
			Ok(c) => c,
			Err(_) => continue,
		};
		let mut defs = Vec::new();
		for line in content.lines() {
			let trimmed = line.trim().to_uppercase();
			if trimmed.starts_with("DEFINE TABLE ")
				|| trimmed.starts_with("DEFINE FIELD ")
				|| trimmed.starts_with("DEFINE INDEX ")
				|| trimmed.starts_with("DEFINE EVENT ")
				|| trimmed.starts_with("DEFINE FUNCTION ")
			{
				defs.push(line.trim().to_string());
			}
		}
		if defs.is_empty() {
			continue;
		}
		file_count += 1;
		schema_text.push_str(&format!("## {rel_path}\n\n```surql\n"));
		for d in &defs {
			schema_text.push_str(d);
			schema_text.push('\n');
		}
		schema_text.push_str("```\n\n");
	}
	if !schema_text.is_empty() {
		schema_text = format!("*{file_count} schema file(s)*\n\n{schema_text}");
	}

	// Write to project root
	let manifest_dir = root.join("surql-lsp-out");
	if std::fs::create_dir_all(&manifest_dir).is_ok() {
		let _ = std::fs::write(
			manifest_dir.join("files.json"),
			serde_json::to_string_pretty(&files).unwrap_or_else(|_| "[]".to_string()),
		);
		let _ = std::fs::write(manifest_dir.join("schema.md"), &schema_text);
	}

	// Write to Zed extension work dir (WASM reads from ".")
	if let Ok(home) = std::env::var("HOME") {
		let zed_ext_dir = std::path::Path::new(&home)
			.join("Library/Application Support/Zed/extensions/work/surrealql");
		if zed_ext_dir.exists() {
			let _ = std::fs::write(zed_ext_dir.join("schema.md"), &schema_text);
			tracing::info!("Schema cache written to Zed extension dir");
		}
	}
}

fn collect_manifest_files(base: &std::path::Path, dir: &std::path::Path, out: &mut Vec<String>) {
	let entries = match std::fs::read_dir(dir) {
		Ok(e) => e,
		Err(_) => return,
	};
	for entry in entries.filter_map(|e| e.ok()) {
		let path = entry.path();
		if path.is_dir() {
			let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
			if matches!(
				name,
				"target"
					| "node_modules"
					| ".git" | "build"
					| "fixtures" | "dist"
					| ".cache" | "surql-lsp-out"
			) || name.starts_with('.')
			{
				continue;
			}
			collect_manifest_files(base, &path, out);
		} else if path.extension().is_some_and(|ext| ext == "surql")
			&& let Ok(rel) = path.strip_prefix(base)
		{
			out.push(rel.to_string_lossy().to_string());
		}
	}
}

/// Build a hierarchical DocumentSymbol tree from a schema and source text.
///
/// Tables become parent symbols (Class) with fields (Field), indexes (Key),
/// and events (Event) as children. Functions and params are top-level symbols.
#[allow(deprecated)] // DocumentSymbol::deprecated field is required but deprecated in LSP spec
pub(crate) fn build_document_symbols(
	source: &str,
	schema: &surql_parser::SchemaGraph,
) -> Option<DocumentSymbolResponse> {
	let mut symbols: Vec<DocumentSymbol> = Vec::new();

	let mut table_names: Vec<&str> = schema.table_names().collect();
	table_names.sort();

	for table_name in &table_names {
		let table = match schema.table(table_name) {
			Some(t) => t,
			None => continue,
		};

		let table_range = find_define_statement_range(source, "TABLE", table_name);
		let mut children: Vec<DocumentSymbol> = Vec::new();

		for field in &table.fields {
			let field_range = find_define_on_table_range(source, "FIELD", &field.name, table_name);
			children.push(DocumentSymbol {
				name: field.name.clone(),
				detail: field.kind.clone(),
				kind: SymbolKind::FIELD,
				tags: None,
				deprecated: None,
				range: field_range,
				selection_range: field_range,
				children: None,
			});
		}

		for index in &table.indexes {
			let index_range = find_define_on_table_range(source, "INDEX", &index.name, table_name);
			let detail = if index.unique {
				Some("UNIQUE".to_string())
			} else {
				None
			};
			children.push(DocumentSymbol {
				name: index.name.clone(),
				detail,
				kind: SymbolKind::KEY,
				tags: None,
				deprecated: None,
				range: index_range,
				selection_range: index_range,
				children: None,
			});
		}

		for event in &table.events {
			let event_range = find_define_on_table_range(source, "EVENT", &event.name, table_name);
			children.push(DocumentSymbol {
				name: event.name.clone(),
				detail: None,
				kind: SymbolKind::EVENT,
				tags: None,
				deprecated: None,
				range: event_range,
				selection_range: event_range,
				children: None,
			});
		}

		let children_opt = if children.is_empty() {
			None
		} else {
			Some(children)
		};

		let schema_detail = if table.full {
			"SCHEMAFULL"
		} else {
			"SCHEMALESS"
		};

		symbols.push(DocumentSymbol {
			name: table_name.to_string(),
			detail: Some(schema_detail.to_string()),
			kind: SymbolKind::CLASS,
			tags: None,
			deprecated: None,
			range: table_range,
			selection_range: table_range,
			children: children_opt,
		});
	}

	let mut fn_names: Vec<&str> = schema.function_names().collect();
	fn_names.sort();

	for fn_name in &fn_names {
		let func = match schema.function(fn_name) {
			Some(f) => f,
			None => continue,
		};
		let fn_range = find_define_statement_range(source, "FUNCTION", &format!("fn::{fn_name}"));
		let sig = func
			.args
			.iter()
			.map(|(n, t)| format!("{n}: {t}"))
			.collect::<Vec<_>>()
			.join(", ");
		let detail = func
			.returns
			.as_ref()
			.map(|r| format!("({sig}) -> {r}"))
			.unwrap_or_else(|| format!("({sig})"));
		symbols.push(DocumentSymbol {
			name: format!("fn::{fn_name}"),
			detail: Some(detail),
			kind: SymbolKind::FUNCTION,
			tags: None,
			deprecated: None,
			range: fn_range,
			selection_range: fn_range,
			children: None,
		});
	}

	let mut param_names: Vec<&str> = schema.param_names().collect();
	param_names.sort();

	for param_name in &param_names {
		let param_range = find_define_statement_range(source, "PARAM", &format!("${param_name}"));
		symbols.push(DocumentSymbol {
			name: format!("${param_name}"),
			detail: None,
			kind: SymbolKind::VARIABLE,
			tags: None,
			deprecated: None,
			range: param_range,
			selection_range: param_range,
			children: None,
		});
	}

	if symbols.is_empty() {
		None
	} else {
		Some(DocumentSymbolResponse::Nested(symbols))
	}
}

/// Build CodeLens annotations for DEFINE TABLE statements showing summary info.
pub(crate) fn build_code_lenses(
	source: &str,
	schema: &surql_parser::SchemaGraph,
) -> Option<Vec<CodeLens>> {
	let mut lenses: Vec<CodeLens> = Vec::new();

	let mut table_names: Vec<&str> = schema.table_names().collect();
	table_names.sort();

	for table_name in &table_names {
		let table = match schema.table(table_name) {
			Some(t) => t,
			None => continue,
		};

		let range = find_define_statement_range(source, "TABLE", table_name);

		let mut parts: Vec<String> = Vec::new();

		let field_count = table.fields.len();
		if field_count > 0 {
			parts.push(format!(
				"{field_count} field{}",
				if field_count == 1 { "" } else { "s" }
			));
		}

		let index_count = table.indexes.len();
		if index_count > 0 {
			parts.push(format!(
				"{index_count} index{}",
				if index_count == 1 { "" } else { "es" }
			));
		}

		let event_count = table.events.len();
		if event_count > 0 {
			parts.push(format!(
				"{event_count} event{}",
				if event_count == 1 { "" } else { "s" }
			));
		}

		let mut outgoing: Vec<String> = Vec::new();
		for field in &table.fields {
			for link in &field.record_links {
				if !outgoing.contains(link) {
					outgoing.push(link.clone());
				}
			}
		}
		outgoing.sort();
		for out in &outgoing {
			parts.push(format!("\u{2192}{out}"));
		}

		let mut incoming: Vec<String> = Vec::new();
		for other_name in &table_names {
			if *other_name == *table_name {
				continue;
			}
			if let Some(other_table) = schema.table(other_name) {
				for field in &other_table.fields {
					if field.record_links.contains(&table_name.to_string())
						&& !incoming.contains(&other_name.to_string())
					{
						incoming.push(other_name.to_string());
					}
				}
			}
		}
		incoming.sort();
		for inc in &incoming {
			parts.push(format!("\u{2190}{inc}"));
		}

		if parts.is_empty() {
			continue;
		}

		let label = parts.join(" \u{00B7} ");

		lenses.push(CodeLens {
			range: Range {
				start: range.start,
				end: range.start,
			},
			command: Some(Command {
				title: label,
				command: String::new(),
				arguments: None,
			}),
			data: None,
		});
	}

	if lenses.is_empty() {
		None
	} else {
		Some(lenses)
	}
}

/// Find the line range of a `DEFINE <kind> <name>` statement via line scanning.
fn find_define_statement_range(source: &str, kind: &str, name: &str) -> Range {
	let upper_kind = kind.to_uppercase();
	let upper_name = name.to_uppercase();
	let search_variants = vec![
		format!("DEFINE {upper_kind} {upper_name}"),
		format!("DEFINE {upper_kind} IF NOT EXISTS {upper_name}"),
		format!("DEFINE {upper_kind} OVERWRITE {upper_name}"),
	];
	for (line_num, line) in source.lines().enumerate() {
		let upper_line = line.to_uppercase();
		let trimmed = upper_line.trim();
		for variant in &search_variants {
			if trimmed.starts_with(variant.as_str()) {
				let line_start = Position {
					line: line_num as u32,
					character: 0,
				};
				let line_end = Position {
					line: line_num as u32,
					character: line.len() as u32,
				};
				return Range {
					start: line_start,
					end: line_end,
				};
			}
		}
	}
	Range {
		start: Position {
			line: 0,
			character: 0,
		},
		end: Position {
			line: 0,
			character: 0,
		},
	}
}

/// Find the line range of a `DEFINE <kind> <name> ON [TABLE] <table>` statement.
fn find_define_on_table_range(source: &str, kind: &str, name: &str, table: &str) -> Range {
	let upper_kind = kind.to_uppercase();
	let upper_name = name.to_uppercase();
	let upper_table = table.to_uppercase();
	for (line_num, line) in source.lines().enumerate() {
		let upper_line = line.to_uppercase();
		let trimmed = upper_line.trim();
		if trimmed.starts_with(&format!("DEFINE {upper_kind}"))
			&& trimmed.contains(&upper_name)
			&& (trimmed.contains(&format!("ON {upper_table}"))
				|| trimmed.contains(&format!("ON TABLE {upper_table}")))
		{
			let line_start = Position {
				line: line_num as u32,
				character: 0,
			};
			let line_end = Position {
				line: line_num as u32,
				character: line.len() as u32,
			};
			return Range {
				start: line_start,
				end: line_end,
			};
		}
	}
	find_define_statement_range(source, "TABLE", table)
}

/// Convert a byte offset in source text to an LSP Position (0-indexed line/col).
pub(crate) fn byte_offset_to_position(source: &str, offset: usize) -> Position {
	let offset = offset.min(source.len());
	let before = &source[..offset];
	let line = before.matches('\n').count() as u32;
	let col = before.rfind('\n').map(|i| offset - i - 1).unwrap_or(offset) as u32;
	Position {
		line,
		character: col,
	}
}

/// Extract the word (identifier) at a cursor position, with its range.
pub(crate) fn word_and_range_at_position(
	source: &str,
	position: Position,
) -> (String, Option<Range>) {
	let line = match source.lines().nth(position.line as usize) {
		Some(l) => l,
		None => return (String::new(), None),
	};
	let col = position.character as usize;
	let bytes = line.as_bytes();

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
		let word = line[start..end].to_string();
		let range = Range {
			start: Position {
				line: position.line,
				character: start as u32,
			},
			end: Position {
				line: position.line,
				character: end as u32,
			},
		};
		(word, Some(range))
	} else {
		(String::new(), None)
	}
}

/// Extract the word (identifier) at a cursor position.
pub(crate) fn word_at_position(source: &str, position: Position) -> String {
	word_and_range_at_position(source, position).0
}

/// SurrealQL type documentation for hover.
fn type_documentation(word: &str, schema: &surql_parser::SchemaGraph) -> Option<String> {
	let lower = word.to_lowercase();
	let doc = match lower.as_str() {
		"string" => {
			"**string** — UTF-8 text\n\n```surql\nDEFINE FIELD name ON user TYPE string\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/simple#string)"
		}
		"int" => {
			"**int** — 64-bit signed integer\n\n```surql\nDEFINE FIELD age ON user TYPE int\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/simple#int)"
		}
		"float" => {
			"**float** — 64-bit floating point\n\n```surql\nDEFINE FIELD score ON user TYPE float\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/simple#float)"
		}
		"decimal" => {
			"**decimal** — Arbitrary precision decimal\n\n```surql\nDEFINE FIELD price ON product TYPE decimal\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/simple#decimal)"
		}
		"number" => {
			"**number** — Any numeric type (int, float, or decimal)\n\n```surql\nDEFINE FIELD value ON data TYPE number\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/simple#number)"
		}
		"bool" => {
			"**bool** — Boolean (true/false)\n\n```surql\nDEFINE FIELD active ON user TYPE bool DEFAULT true\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/simple#bool)"
		}
		"datetime" => {
			"**datetime** — ISO 8601 timestamp\n\n```surql\nDEFINE FIELD created_at ON user TYPE datetime DEFAULT time::now()\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/simple#datetime)"
		}
		"duration" => {
			"**duration** — Time span (e.g., 1h, 30m, 7d)\n\n```surql\nDEFINE FIELD ttl ON cache TYPE duration\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/simple#duration)"
		}
		"object" => {
			"**object** — JSON-like object (key-value map)\n\n```surql\nDEFINE FIELD settings ON user TYPE object DEFAULT {}\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/simple#object)"
		}
		"array" => {
			"**array** — Ordered collection. Parameterized: `array<string>`\n\n```surql\nDEFINE FIELD tags ON post TYPE array<string> DEFAULT []\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/simple#array)"
		}
		"set" => {
			"**set** — Unique collection (no duplicates). Parameterized: `set<string>`\n\n```surql\nDEFINE FIELD roles ON user TYPE set<string>\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/simple#set)"
		}
		"option" => {
			"**option** — Nullable type. `option<T>` means the field can be NONE\n\n```surql\nDEFINE FIELD bio ON user TYPE option<string>\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/simple#option)"
		}
		"record" => {
			// For record<user>, show the target table's fields
			let tables: Vec<_> = schema.table_names().collect();
			let table_list = if tables.is_empty() {
				String::new()
			} else {
				format!(
					"\n\nTables in schema: `{}`",
					tables.into_iter().collect::<Vec<_>>().join("`, `")
				)
			};
			return Some(format!(
				"**record** — Link to another record. Parameterized: `record<table>`\n\n\
				 ```surql\nDEFINE FIELD author ON post TYPE record<user>\n```\n\n\
				 The linked record can be fetched with `FETCH`.{table_list}\n\n\
				 [Docs](https://surrealdb.com/docs/surrealql/datamodel/simple#record)"
			));
		}
		"uuid" => {
			"**uuid** — Universally unique identifier\n\n```surql\nDEFINE FIELD id ON user TYPE uuid DEFAULT rand::uuid::v4()\n```\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/simple#uuid)"
		}
		"bytes" => {
			"**bytes** — Binary data\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/simple#bytes)"
		}
		"geometry" => {
			"**geometry** — GeoJSON geometry (point, line, polygon, etc.)\n\n[Docs](https://surrealdb.com/docs/surrealql/datamodel/geometries)"
		}
		"any" => {
			"**any** — Accepts any type (no type constraint)\n\n```surql\nDEFINE FIELD data ON flexible_table TYPE any\n```"
		}
		_ => return None,
	};
	Some(doc.to_string())
}

/// Find all occurrences of a word in source text and add them as Locations.
pub(crate) fn find_word_occurrences(
	source: &str,
	word: &str,
	uri: &Url,
	locations: &mut Vec<Location>,
) {
	let bytes = word.as_bytes();
	for (line_num, line) in source.lines().enumerate() {
		let line_bytes = line.as_bytes();
		let mut col = 0;
		while col + bytes.len() <= line_bytes.len() {
			if line_bytes[col..].starts_with(bytes) {
				let before_ok = col == 0
					|| !(line_bytes[col - 1].is_ascii_alphanumeric()
						|| line_bytes[col - 1] == b'_'
						|| line_bytes[col - 1] == b':');
				let after_pos = col + bytes.len();
				let after_ok = after_pos >= line_bytes.len()
					|| !(line_bytes[after_pos].is_ascii_alphanumeric()
						|| line_bytes[after_pos] == b'_'
						|| line_bytes[after_pos] == b':');
				if before_ok && after_ok {
					locations.push(Location {
						uri: uri.clone(),
						range: Range {
							start: Position {
								line: line_num as u32,
								character: col as u32,
							},
							end: Position {
								line: line_num as u32,
								character: (col + bytes.len()) as u32,
							},
						},
					});
				}
			}
			col += 1;
		}
	}
}

/// Tokenize an embedded SurrealQL region into LSP semantic tokens.
///
/// Uses the SurrealQL lexer and maps token kinds to semantic token types.
/// Positions are relative to the host file (not the region).
fn tokenize_surql_region(
	host_source: &str,
	region: &embedded::EmbeddedRegion,
	tokens: &mut Vec<SemanticToken>,
) {
	use surql_parser::upstream::syn::lexer::Lexer;
	use surql_parser::upstream::syn::token::TokenKind;

	let bytes = region.content.as_bytes();
	if bytes.is_empty() || bytes.len() > u32::MAX as usize {
		return;
	}

	// Lexer may panic on malformed input — catch and skip
	let token_list: Vec<_> = match std::panic::catch_unwind(|| Lexer::new(bytes).collect()) {
		Ok(t) => t,
		Err(_) => return,
	};

	let mut prev_line = region.line;
	let mut prev_col = region.col;

	for token in &token_list {
		let tok_offset = region.offset + token.span.offset as usize;
		let tok_len = token.span.len;
		if tok_len == 0 || tok_offset >= host_source.len() {
			continue;
		}

		let token_type = match token.kind {
			TokenKind::Keyword(_) => 0,  // KEYWORD
			TokenKind::Identifier => 6,  // TYPE (table/field names)
			TokenKind::Parameter => 2,   // VARIABLE
			TokenKind::String(_) => 3,   // STRING
			TokenKind::Digits => 4,      // NUMBER
			TokenKind::Operator(_) => 5, // OPERATOR
			_ => continue,
		};

		let pos = byte_offset_to_position(host_source, tok_offset);
		let delta_line = pos.line.saturating_sub(prev_line);
		let delta_start = if delta_line == 0 {
			pos.character.saturating_sub(prev_col)
		} else {
			pos.character
		};

		tokens.push(SemanticToken {
			delta_line,
			delta_start,
			length: tok_len,
			token_type,
			token_modifiers_bitset: 0,
		});

		prev_line = pos.line;
		prev_col = pos.character;
	}
}

fn format_table_hover(
	name: &str,
	table: &surql_parser::schema_graph::TableDef,
	fields: &[surql_parser::schema_graph::FieldDef],
) -> String {
	let schema_type = if table.full {
		"SCHEMAFULL"
	} else {
		"SCHEMALESS"
	};
	let comment_line = table
		.comment
		.as_ref()
		.map(|c| format!("\n\n*{c}*"))
		.unwrap_or_default();
	let field_list = fields
		.iter()
		.map(|f| {
			let kind = f.kind.as_deref().unwrap_or("any");
			let default = f
				.default
				.as_ref()
				.map(|d| format!(" DEFAULT {d}"))
				.unwrap_or_default();
			let readonly = if f.readonly { " READONLY" } else { "" };
			let comment = f
				.comment
				.as_ref()
				.map(|c| format!("  -- {c}"))
				.unwrap_or_default();
			format!("{} : {kind}{default}{readonly}{comment}", f.name)
		})
		.collect::<Vec<_>>()
		.join("\n");
	format!(
		"```surql\n-- TABLE {name} ({schema_type})\n```\n{comment_line}\n\n\
		 ```surql\n{field_list}\n```"
	)
}

fn format_function_hover(func: &surql_parser::schema_graph::FunctionDef) -> String {
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
	let comment_line = func
		.comment
		.as_ref()
		.map(|c| format!("\n\n*{c}*"))
		.unwrap_or_default();
	format!(
		"**FUNCTION** `fn::{}`{comment_line}\n\n```surql\nfn::{}({args}){ret}\n```",
		func.name, func.name
	)
}

/// SurrealQL keyword documentation for hover.
pub(crate) fn keyword_documentation(word: &str) -> Option<&'static str> {
	match word.to_uppercase().as_str() {
		"SELECT" => Some(
			"**SELECT** — Query data from tables\n\n\
			 ```surql\nSELECT field1, field2 FROM table WHERE condition\n\
			 ORDER BY field LIMIT n\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/select)",
		),
		"CREATE" => Some(
			"**CREATE** — Create a new record\n\n\
			 ```surql\nCREATE table SET field = value\n\
			 CREATE table CONTENT { field: value }\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/create)",
		),
		"UPDATE" => Some(
			"**UPDATE** — Modify existing records\n\n\
			 ```surql\nUPDATE table SET field = value WHERE condition\n\
			 UPDATE table MERGE { field: value }\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/update)",
		),
		"DELETE" => Some(
			"**DELETE** — Remove records\n\n\
			 ```surql\nDELETE table WHERE condition\nDELETE record:id\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/delete)",
		),
		"INSERT" => Some(
			"**INSERT** — Insert records (supports ON DUPLICATE KEY UPDATE)\n\n\
			 ```surql\nINSERT INTO table { field: value }\n\
			 INSERT INTO table [{ ... }, { ... }]\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/insert)",
		),
		"UPSERT" => Some(
			"**UPSERT** — Create or update a record atomically\n\n\
			 ```surql\nUPSERT table SET field = value WHERE condition\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/upsert)",
		),
		"RELATE" => Some(
			"**RELATE** — Create a graph edge between two records\n\n\
			 ```surql\nRELATE from_record->edge_table->to_record\n\
			 SET field = value\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/relate)",
		),
		"DEFINE" => Some(
			"**DEFINE** — Define schema elements\n\n\
			 ```surql\nDEFINE TABLE name SCHEMAFULL\n\
			 DEFINE FIELD name ON table TYPE string\n\
			 DEFINE INDEX name ON table FIELDS field UNIQUE\n\
			 DEFINE FUNCTION fn::name($arg: type) { ... }\n\
			 DEFINE EVENT name ON table WHEN $event = 'CREATE' THEN { ... }\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/define)",
		),
		"REMOVE" => Some(
			"**REMOVE** — Remove schema definitions\n\n\
			 ```surql\nREMOVE TABLE name\nREMOVE FIELD name ON table\n\
			 REMOVE INDEX name ON table\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/remove)",
		),
		"LET" => Some(
			"**LET** — Bind a value to a parameter\n\n\
			 ```surql\nLET $name = expression\n\
			 LET $results = SELECT * FROM table\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/let)",
		),
		"IF" => Some(
			"**IF** — Conditional expression\n\n\
			 ```surql\nIF condition { ... }\n\
			 ELSE IF condition { ... }\n\
			 ELSE { ... }\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/if-else)",
		),
		"FOR" => Some(
			"**FOR** — Iterate over values\n\n\
			 ```surql\nFOR $item IN $array { ... }\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/for)",
		),
		"RETURN" => Some(
			"**RETURN** — Return a value from a block or function\n\n\
			 ```surql\nRETURN expression\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/return)",
		),
		"BEGIN" => Some(
			"**BEGIN** — Start a transaction\n\n\
			 ```surql\nBEGIN TRANSACTION;\n-- statements\nCOMMIT TRANSACTION;\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/begin)",
		),
		"COMMIT" => Some(
			"**COMMIT** — Commit a transaction\n\n\
			 ```surql\nCOMMIT TRANSACTION\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/begin)",
		),
		"CANCEL" => Some(
			"**CANCEL** — Roll back a transaction\n\n\
			 ```surql\nCANCEL TRANSACTION\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/begin)",
		),
		"LIVE" => Some(
			"**LIVE** — Subscribe to real-time changes\n\n\
			 ```surql\nLIVE SELECT * FROM table\n\
			 LIVE SELECT DIFF FROM table\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/live)",
		),
		"KILL" => Some(
			"**KILL** — Stop a live query\n\n\
			 ```surql\nKILL $live_query_id\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/kill)",
		),
		"USE" => Some(
			"**USE** — Switch namespace or database\n\n\
			 ```surql\nUSE NS namespace DB database\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/use)",
		),
		"INFO" => Some(
			"**INFO** — Show database information\n\n\
			 ```surql\nINFO FOR DB\nINFO FOR TABLE name\nINFO FOR ROOT\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/info)",
		),
		"SLEEP" => Some(
			"**SLEEP** — Pause execution\n\n\
			 ```surql\nSLEEP 1s\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/sleep)",
		),
		"THROW" => Some(
			"**THROW** — Throw a custom error\n\n\
			 ```surql\nTHROW 'error message'\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/throw)",
		),
		"SCHEMAFULL" => Some(
			"**SCHEMAFULL** — Only allow defined fields on a table\n\n\
			 ```surql\nDEFINE TABLE name SCHEMAFULL\n```\n\n\
			 Undefined fields are rejected. Use SCHEMALESS for flexible structure.",
		),
		"SCHEMALESS" => Some(
			"**SCHEMALESS** — Allow any fields on a table (default)\n\n\
			 ```surql\nDEFINE TABLE name SCHEMALESS\n```\n\n\
			 Any field can be set. Use SCHEMAFULL for strict structure.",
		),
		"CHANGEFEED" => Some(
			"**CHANGEFEED** — Enable change tracking on a table\n\n\
			 ```surql\nDEFINE TABLE name CHANGEFEED 1d INCLUDE ORIGINAL\n\
			 SHOW CHANGES FOR TABLE name SINCE timestamp\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/define/table)",
		),
		"PERMISSIONS" => Some(
			"**PERMISSIONS** — Set access control on tables/fields\n\n\
			 ```surql\nDEFINE TABLE name PERMISSIONS\n\
			 \tFOR select WHERE published = true\n\
			 \tFOR create, update WHERE $auth.id = author\n\
			 \tFOR delete NONE\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/define/table)",
		),
		"FETCH" => Some(
			"**FETCH** — Eagerly load linked records\n\n\
			 ```surql\nSELECT * FROM post FETCH author, comments\n```\n\n\
			 Replaces record links with their full content.",
		),
		"VALUE" | "VALUES" => Some(
			"**VALUE** / **VALUES** — Return raw value instead of wrapped result\n\n\
			 ```surql\nSELECT VALUE name FROM user\n```\n\n\
			 Returns flat array of values instead of array of objects.",
		),
		"EXPLAIN" => Some(
			"**EXPLAIN** — Show query execution plan\n\n\
			 ```surql\nSELECT * FROM user WHERE age > 18 EXPLAIN FULL\n```\n\n\
			 [Docs](https://surrealdb.com/docs/surrealql/statements/select)",
		),
		"PARALLEL" => Some(
			"**PARALLEL** — Execute query in parallel\n\n\
			 ```surql\nSELECT * FROM user WHERE age > 18 PARALLEL\n```",
		),
		_ => None,
	}
}
