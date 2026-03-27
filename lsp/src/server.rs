//! LSP Backend — implements the Language Server Protocol for SurrealQL.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use dashmap::DashMap;
use surql_parser::SchemaGraph;
use tokio::sync::RwLock;
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
use crate::manifest::{detect_overshift_manifest, write_file_manifest};

// Re-export moved functions for backward compatibility (used by tests)
pub(crate) use crate::dotenv::{detect_monorepo_projects, load_dotenv};
pub(crate) use crate::hover::{
	GraphContext, dotted_path_at_position, format_function_hover, format_nested_field_hover,
	format_table_hover, graph_context_at_position, keyword_documentation, type_documentation,
};
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
	schema: RwLock<Arc<SchemaGraph>>,
	document_schemas: DashMap<Url, SchemaGraph>,
	file_schemas: RwLock<HashMap<PathBuf, SchemaGraph>>,
	workspace_root: RwLock<Option<PathBuf>>,
	manifest_scope: RwLock<Option<(String, String)>>,
	dotenv_scope: RwLock<Option<(String, String)>>,
	format_config: RwLock<formatting::FormatConfig>,
	#[cfg(feature = "embedded-db")]
	embedded: RwLock<Option<crate::embedded_db::DualEngine>>,
}

impl Backend {
	pub fn new(client: Client) -> Self {
		Self {
			client,
			documents: DocumentStore::new(),
			schema: RwLock::new(Arc::new(SchemaGraph::default())),
			document_schemas: DashMap::new(),
			file_schemas: RwLock::new(HashMap::new()),
			workspace_root: RwLock::new(None),
			manifest_scope: RwLock::new(None),
			dotenv_scope: RwLock::new(None),
			format_config: RwLock::new(formatting::FormatConfig::default()),
			#[cfg(feature = "embedded-db")]
			embedded: RwLock::new(None),
		}
	}

	/// Full rebuild: scan all .surql files, populate per-file cache, merge into
	/// workspace schema. Called once on `initialized`.
	///
	/// Filesystem I/O runs on a blocking thread via `spawn_blocking` to avoid
	/// stalling the async runtime. Shows progress in the editor status bar.
	async fn rebuild_schema(&self) {
		let root = self.workspace_root.read().await.clone();
		if let Some(root) = root {
			let token = NumberOrString::String("surql-schema-rebuild".into());
			let _ = self
				.client
				.send_request::<request::WorkDoneProgressCreate>(WorkDoneProgressCreateParams {
					token: token.clone(),
				})
				.await;

			self.client
				.send_notification::<notification::Progress>(ProgressParams {
					token: token.clone(),
					value: ProgressParamsValue::WorkDone(WorkDoneProgress::Begin(
						WorkDoneProgressBegin {
							title: "SurrealQL".into(),
							message: Some("Scanning .surql files...".into()),
							cancellable: Some(false),
							percentage: Some(0),
						},
					)),
				})
				.await;

			let build_result = tokio::task::spawn_blocking(move || {
				let per_file = SchemaGraph::from_files_per_file(&root);
				(root, per_file)
			})
			.await;

			let (root, per_file_result) = match build_result {
				Ok(pair) => pair,
				Err(e) => {
					tracing::error!("rebuild_schema spawn_blocking panicked: {e}");
					self.send_progress_end(&token, "Schema build failed").await;
					return;
				}
			};
			match per_file_result {
				Ok(per_file) => {
					*self.file_schemas.write().await = per_file;
					let merged = self.merge_file_schemas().await;
					let table_count = merged.table_names().count();
					let fn_count = merged.function_names().count();
					tracing::info!(
						"Schema rebuilt: {} tables, {} functions from {}",
						table_count,
						fn_count,
						root.display()
					);

					self.client
						.send_notification::<notification::Progress>(ProgressParams {
							token: token.clone(),
							value: ProgressParamsValue::WorkDone(WorkDoneProgress::Report(
								WorkDoneProgressReport {
									message: Some(format!(
										"Building manifest ({table_count} tables, {fn_count} functions)..."
									)),
									cancellable: Some(false),
									percentage: Some(70),
								},
							)),
						})
						.await;

					let manifest_schema = Arc::new(merged);
					let manifest_root = root.clone();
					let manifest_ref = Arc::clone(&manifest_schema);
					let manifest_handle = tokio::task::spawn_blocking(move || {
						write_file_manifest(&manifest_root, &manifest_ref);
					});
					tokio::spawn(async move {
						if let Err(e) = manifest_handle.await {
							tracing::error!("write_file_manifest panicked: {e}");
						}
					});

					self.send_progress_end(
						&token,
						&format!("{table_count} tables, {fn_count} functions"),
					)
					.await;
				}
				Err(e) => {
					tracing::warn!("Failed to rebuild schema: {e}");
					let manifest_root = root.clone();
					let manifest_handle = tokio::task::spawn_blocking(move || {
						write_file_manifest(&manifest_root, &SchemaGraph::default());
					});
					tokio::spawn(async move {
						if let Err(e) = manifest_handle.await {
							tracing::error!("write_file_manifest panicked: {e}");
						}
					});
					self.send_progress_end(&token, "Schema build failed").await;
				}
			}
		} else {
			tracing::debug!("rebuild_schema: no workspace root set");
		}
	}

	/// Incrementally rebuild the schema for a single file that changed.
	///
	/// Re-parses only `path`, updates the per-file cache, then re-merges all
	/// cached schemas. O(1) parse + O(n) merge where n = file count (merge is
	/// cheap HashMap extends, no I/O or parsing).
	async fn rebuild_file_schema(&self, path: PathBuf) {
		let file_path = path.clone();
		let build_result =
			tokio::task::spawn_blocking(move || SchemaGraph::from_single_file(&file_path)).await;

		match build_result {
			Ok(Some(graph)) => {
				self.file_schemas.write().await.insert(path, graph);
			}
			Ok(None) => {
				// File unreadable/unparsable — remove stale entry
				self.file_schemas.write().await.remove(&path);
			}
			Err(e) => {
				tracing::error!("rebuild_file_schema spawn_blocking panicked: {e}");
				return;
			}
		}

		let merged = self.merge_file_schemas().await;

		// Write manifest in the background
		if let Some(root) = self.workspace_root.read().await.clone() {
			let manifest_schema = Arc::new(merged);
			let manifest_ref = Arc::clone(&manifest_schema);
			let manifest_handle = tokio::task::spawn_blocking(move || {
				write_file_manifest(&root, &manifest_ref);
			});
			tokio::spawn(async move {
				if let Err(e) = manifest_handle.await {
					tracing::error!("write_file_manifest panicked: {e}");
				}
			});
		}
	}

	/// Merge all per-file schemas into the workspace-level schema.
	///
	/// Returns a clone of the merged schema for callers that need it (e.g.
	/// to write the manifest). The merged schema is also stored in `self.schema`.
	async fn merge_file_schemas(&self) -> SchemaGraph {
		let file_schemas = self.file_schemas.read().await;
		let mut merged = SchemaGraph::default();
		for sg in file_schemas.values() {
			merged.merge(sg.clone());
		}
		let schema_arc = Arc::new(merged.clone());
		drop(file_schemas);
		*self.schema.write().await = schema_arc;
		merged
	}

	async fn send_progress_end(&self, token: &NumberOrString, message: &str) {
		self.client
			.send_notification::<notification::Progress>(ProgressParams {
				token: token.clone(),
				value: ProgressParamsValue::WorkDone(WorkDoneProgress::End(WorkDoneProgressEnd {
					message: Some(message.into()),
				})),
			})
			.await;
	}

	/// Get the effective schema for a document: workspace schema + document overlay.
	/// Applies NS/DB scope filtering based on the current document's context.
	///
	/// Clones the `Arc<SchemaGraph>` (cheap ref-count bump). Real cloning only
	/// happens when scoping or merging is needed.
	async fn effective_schema(&self, uri: &Url) -> SchemaGraph {
		let schema_arc = Arc::clone(&*self.schema.read().await);

		// Determine current file's NS/DB scope:
		// 1. From USE statement in the file (document schema)
		// 2. From manifest.toml (overshift project)
		// 3. From .env file (SURREALDB_NS / SURREALDB_DB)
		// 4. Default (None, None) = see all unscoped tables
		let from_doc = self.document_schemas.get(uri).and_then(|ds| {
			ds.table_names()
				.next()
				.and_then(|tn| ds.table(tn))
				.and_then(|t| {
					if t.ns.is_some() || t.db.is_some() {
						Some((t.ns.clone(), t.db.clone()))
					} else {
						None
					}
				})
		});
		let (file_ns, file_db) = match from_doc {
			Some(scope) => scope,
			None => {
				let manifest = self.manifest_scope.read().await;
				if let Some((ns, db)) = manifest.as_ref() {
					(Some(ns.clone()), Some(db.clone()))
				} else {
					let dotenv = self.dotenv_scope.read().await;
					dotenv
						.as_ref()
						.map(|(ns, db)| (Some(ns.clone()), Some(db.clone())))
						.unwrap_or((None, None))
				}
			}
		};

		let needs_scope = file_ns.is_some() || file_db.is_some();
		let has_doc_schema = self.document_schemas.get(uri).is_some();

		if !needs_scope && !has_doc_schema {
			// Fast path: no scoping or merging needed, return owned clone via Arc
			return (*schema_arc).clone();
		}

		let mut schema = schema_arc.scoped(file_ns.as_deref(), file_db.as_deref());

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

	/// Collect all .surql files in the workspace: open documents merged with
	/// files on disk. Open documents take priority (unsaved changes).
	async fn collect_workspace_surql_files(&self) -> Vec<(Url, String)> {
		let mut file_map: std::collections::HashMap<Url, String> = std::collections::HashMap::new();

		// Start with all open .surql documents
		for (doc_uri, doc_source) in self.documents.all() {
			if doc_uri.path().ends_with(".surql") {
				file_map.insert(doc_uri, doc_source);
			}
		}

		// Add workspace files from disk that aren't already open
		if let Some(root) = self.workspace_root.read().await.clone() {
			let disk_files = tokio::task::spawn_blocking(move || {
				let mut paths = Vec::new();
				surql_parser::collect_surql_files(&root, &mut paths);
				let mut result = Vec::new();
				for path in paths {
					if let Ok(uri) = Url::from_file_path(&path)
						&& let Ok(content) = std::fs::read_to_string(&path)
					{
						result.push((uri, content));
					}
				}
				result
			})
			.await
			.unwrap_or_default();

			for (uri, content) in disk_files {
				file_map.entry(uri).or_insert(content);
			}
		}

		file_map.into_iter().collect()
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

			// Schema-aware diagnostics
			let schema = self.effective_schema(&uri).await;
			if schema.table_names().count() > 0 {
				// Warn about undefined table references in DML
				for table_ref in context::extract_table_references(&source) {
					if schema.table(&table_ref.name).is_none()
						&& !line_has_suppress(&source, table_ref.line, "undefined-table")
					{
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

				// Cross-file: warn about record links to undefined tables
				// Use raw workspace schema (not scoped) to avoid false positives
				let ws_schema = self.schema.read().await;
				if let Some(doc_schema) = self.document_schemas.get(&uri) {
					for table_name in doc_schema.table_names() {
						for field in doc_schema.fields_of(table_name) {
							for link in &field.record_links {
								if ws_schema.table(link).is_none()
									&& schema.table(link).is_none()
									&& let Some(range) = find_record_link_range(&source, link)
									&& !line_has_suppress(
										&source,
										range.start.line,
										"undefined-table",
									) {
									all_diagnostics.push(Diagnostic {
										range,
										severity: Some(DiagnosticSeverity::WARNING),
										source: Some("surql-schema".into()),
										message: format!(
											"Record link to '{}' but table is not defined",
											link
										),
										..Default::default()
									});
								}
							}
						}
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
				Ok(Ok(_)) => continue,
				Err(e) => {
					tracing::error!("Lexer panicked during Rust diagnostics: {e:?}");
					continue;
				}
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
		{
			let manifest_path = path.clone();
			tokio::task::spawn_blocking(move || {
				write_file_manifest(&manifest_path, &SchemaGraph::default());
			});
			*self.format_config.write().await = formatting::load_config_from_workspace(&path);
			*self.workspace_root.write().await = Some(path);
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
				code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
				rename_provider: Some(OneOf::Right(RenameOptions {
					prepare_provider: Some(true),
					work_done_progress_options: WorkDoneProgressOptions {
						work_done_progress: None,
					},
				})),
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

		// Auto-detect overshift manifest.toml for NS/DB context
		{
			let root = self.workspace_root.read().await;
			if let Some(root) = root.as_ref() {
				if let Some(scope) = detect_overshift_manifest(root) {
					*self.manifest_scope.write().await = Some(scope);
				}

				if let Some((_url, ns, db)) = load_dotenv(root) {
					tracing::info!("Loaded .env scope: NS={ns}, DB={db}");
					*self.dotenv_scope.write().await = Some((ns, db));
				}

				let monorepo_roots = detect_monorepo_projects(root);
				if monorepo_roots.len() > 1 {
					let listing = monorepo_roots
						.iter()
						.filter_map(|p| p.strip_prefix(root).ok())
						.map(|p| p.display().to_string())
						.collect::<Vec<_>>()
						.join(", ");
					tracing::warn!(
						"Multiple SurrealDB projects detected: {listing}. \
						 Open each as a workspace for best results."
					);
					self.client
						.show_message(
							MessageType::WARNING,
							format!(
								"Multiple SurrealDB projects detected: {listing}. \
								 Open each as a separate workspace for best results."
							),
						)
						.await;
				}
			}
		}

		self.rebuild_schema().await;

		// Re-publish diagnostics for all open docs now that schema is built
		for doc_uri in self.documents.all_uris() {
			self.publish_diagnostics(doc_uri).await;
		}

		#[cfg(feature = "embedded-db")]
		{
			let workspace_path = self.workspace_root.read().await.clone();
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
		let uri = params.text_document.uri.clone();
		if let Ok(path) = uri.to_file_path()
			&& path.extension().is_some_and(|ext| ext == "surql")
		{
			self.rebuild_file_schema(path).await;
		}

		#[cfg(feature = "embedded-db")]
		{
			if let Some(ref engine) = *self.embedded.read().await {
				let workspace_path = self.workspace_root.read().await.clone();
				if let Some(root) = workspace_path
					&& let Err(e) = engine.apply_migrations(&root).await
				{
					tracing::warn!("Failed to reapply migrations on save: {e}");
				}
			}
		}

		// Re-publish diagnostics for ALL open documents, not just the saved one,
		// because schema changes in one file can affect diagnostics in other files.
		for doc_uri in self.documents.all_uris() {
			self.publish_diagnostics(doc_uri).await;
		}
	}

	async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
		let uri = &params.text_document.uri;
		let source = match self.documents.get(uri) {
			Some(s) => s,
			None => return Ok(None),
		};
		let config = self.format_config.read().await.clone();
		Ok(formatting::format_document(&source, &config))
	}

	async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
		let uri = &params.text_document_position.text_document.uri;
		let position = params.text_document_position.position;
		let source = match self.documents.get(uri) {
			Some(s) => s,
			None => return Ok(None),
		};
		let schema = self.effective_schema(uri).await;
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
			let schema = self.effective_schema(uri).await;
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

		let schema = self.effective_schema(uri).await;

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

		// Graph path hover: ->edge->, ->target, ->target.field
		if let Some(ctx) = graph_context_at_position(&source, position) {
			match ctx {
				GraphContext::EdgeTable(name) | GraphContext::TargetTable(name) => {
					if let Some(table) = schema.table(&name) {
						let content = format_table_hover(&name, table, schema.fields_of(&name));
						return Ok(Some(Hover {
							contents: HoverContents::Markup(MarkupContent {
								kind: MarkupKind::Markdown,
								value: content,
							}),
							range: word_range,
						}));
					}
				}
				GraphContext::FieldOnTarget { table, field } => {
					if let Some(f) = schema.field_on(&table, &field) {
						let content = format_nested_field_hover(&table, &field, f);
						return Ok(Some(Hover {
							contents: HoverContents::Markup(MarkupContent {
								kind: MarkupKind::Markdown,
								value: content,
							}),
							range: word_range,
						}));
					}
				}
			}
		}

		// Nested field hover: settings.theme -> resolve through schema
		if let Some(table_name) = context::table_context_at_position(&source, position) {
			let dotted = dotted_path_at_position(&source, position);
			if dotted.contains('.') {
				// Try the full dotted path as a field name (e.g. "settings.theme")
				if let Some(f) = schema.field_on(&table_name, &dotted) {
					let content = format_nested_field_hover(&table_name, &dotted, f);
					return Ok(Some(Hover {
						contents: HoverContents::Markup(MarkupContent {
							kind: MarkupKind::Markdown,
							value: content,
						}),
						range: word_range,
					}));
				}
			}
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

		// SurrealQL keyword documentation — check for compound keywords (DEFINE TABLE, etc.)
		let compound_word = detect_compound_keyword(&source, position, &word);
		let lookup = compound_word.as_deref().unwrap_or(&word);
		if let Some(doc) = keyword_documentation(lookup) {
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
		let schema = self.effective_schema(uri).await;
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

		let schema = self.effective_schema(uri).await;

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

		let symbol = context::classify_symbol_at_position(&source, position);

		// Collect all document sources: open documents + workspace .surql files
		let mut all_sources = self.documents.all();
		let open_uris: std::collections::HashSet<Url> =
			all_sources.iter().map(|(u, _)| u.clone()).collect();
		if let Some(root) = self.workspace_root.read().await.as_ref() {
			for (ws_uri, ws_source) in context::collect_workspace_surql_sources(root) {
				if !open_uris.contains(&ws_uri) {
					all_sources.push((ws_uri, ws_source));
				}
			}
		}

		let mut locations = Vec::new();

		match symbol {
			Some(context::SymbolKind::Table(ref table_name)) => {
				for (doc_uri, doc_source) in &all_sources {
					let refs = context::extract_all_table_occurrences(doc_source, table_name);
					for r in refs {
						locations.push(Location {
							uri: doc_uri.clone(),
							range: Range {
								start: Position {
									line: r.line,
									character: r.col,
								},
								end: Position {
									line: r.line,
									character: r.col + r.len,
								},
							},
						});
					}
				}
			}
			Some(context::SymbolKind::Field {
				ref table,
				ref field,
			}) => {
				for (doc_uri, doc_source) in &all_sources {
					let refs = context::find_field_references(doc_source, table, field);
					for r in refs {
						locations.push(Location {
							uri: doc_uri.clone(),
							range: Range {
								start: Position {
									line: r.line,
									character: r.col,
								},
								end: Position {
									line: r.line,
									character: r.col + r.len,
								},
							},
						});
					}
				}
			}
			Some(context::SymbolKind::Function(ref fn_name)) => {
				for (doc_uri, doc_source) in &all_sources {
					let refs = context::find_function_references_in(doc_source, fn_name);
					for r in refs {
						locations.push(Location {
							uri: doc_uri.clone(),
							range: Range {
								start: Position {
									line: r.line,
									character: r.col,
								},
								end: Position {
									line: r.line,
									character: r.col + r.len,
								},
							},
						});
					}
				}
			}
			_ => {
				let word = match &symbol {
					Some(context::SymbolKind::Unknown(w)) => w.clone(),
					_ => word_at_position(&source, position),
				};
				if word.is_empty() {
					return Ok(None);
				}
				for (doc_uri, doc_source) in &all_sources {
					find_word_occurrences(doc_source, &word, doc_uri, &mut locations);
				}
			}
		}

		if locations.is_empty() {
			Ok(None)
		} else {
			Ok(Some(locations))
		}
	}

	async fn prepare_rename(
		&self,
		params: TextDocumentPositionParams,
	) -> Result<Option<PrepareRenameResponse>> {
		let uri = &params.text_document.uri;
		if Self::is_rust_file(uri) {
			return Ok(None);
		}
		let source = match self.documents.get(uri) {
			Some(s) => s,
			None => return Ok(None),
		};

		let (word, range) = word_and_range_at_position(&source, params.position);
		if word.is_empty() {
			return Ok(None);
		}

		let schema = self.effective_schema(uri).await;
		let is_table = schema.table(&word).is_some();
		let fn_name = word.strip_prefix("fn::").unwrap_or(&word);
		let is_function = schema.function(fn_name).is_some();

		if !is_table && !is_function {
			return Ok(None);
		}

		match range {
			Some(r) => Ok(Some(PrepareRenameResponse::Range(r))),
			None => Ok(None),
		}
	}

	async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
		let uri = &params.text_document_position.text_document.uri;
		if Self::is_rust_file(uri) {
			return Ok(None);
		}
		let source = match self.documents.get(uri) {
			Some(s) => s,
			None => return Ok(None),
		};

		let old_name = word_at_position(&source, params.text_document_position.position);
		if old_name.is_empty() {
			return Ok(None);
		}

		let new_name = &params.new_name;
		if new_name.is_empty() || old_name == *new_name {
			return Ok(None);
		}

		let schema = self.effective_schema(uri).await;
		let is_table = schema.table(&old_name).is_some();
		let fn_name = old_name.strip_prefix("fn::").unwrap_or(&old_name);
		let is_function = schema.function(fn_name).is_some();

		if !is_table && !is_function {
			return Ok(None);
		}

		let mut changes: std::collections::HashMap<Url, Vec<TextEdit>> =
			std::collections::HashMap::new();

		if is_table {
			// Collect all .surql file contents: open documents + workspace files on disk
			let file_contents = self.collect_workspace_surql_files().await;

			for (file_uri, content) in &file_contents {
				let occurrences = context::extract_all_table_occurrences(content, &old_name);
				if occurrences.is_empty() {
					continue;
				}
				let edits: Vec<TextEdit> = occurrences
					.iter()
					.map(|occ| TextEdit {
						range: Range {
							start: Position {
								line: occ.line,
								character: occ.col,
							},
							end: Position {
								line: occ.line,
								character: occ.col + occ.len,
							},
						},
						new_text: new_name.clone(),
					})
					.collect();
				changes.insert(file_uri.clone(), edits);
			}
		} else if is_function {
			let full_fn_name = if old_name.starts_with("fn::") {
				old_name.clone()
			} else {
				format!("fn::{old_name}")
			};
			let full_new_name = if new_name.starts_with("fn::") {
				new_name.clone()
			} else {
				format!("fn::{new_name}")
			};
			let file_contents = self.collect_workspace_surql_files().await;
			for (file_uri, content) in &file_contents {
				let refs = context::find_function_references_in(content, &full_fn_name);
				if refs.is_empty() {
					continue;
				}
				let edits: Vec<TextEdit> = refs
					.iter()
					.map(|r| TextEdit {
						range: Range {
							start: Position {
								line: r.line,
								character: r.col,
							},
							end: Position {
								line: r.line,
								character: r.col + r.len,
							},
						},
						new_text: full_new_name.clone(),
					})
					.collect();
				changes.insert(file_uri.clone(), edits);
			}
		}

		if changes.is_empty() {
			return Ok(None);
		}

		Ok(Some(WorkspaceEdit {
			changes: Some(changes),
			..Default::default()
		}))
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

	async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
		let uri = &params.text_document.uri;
		let source = match self.documents.get(uri) {
			Some(s) => s,
			None => return Ok(None),
		};

		let mut actions = Vec::new();

		for diag in &params.context.diagnostics {
			let suppress_code = if diag.message.contains("is not defined in workspace") {
				Some("undefined-table")
			} else if diag.message.contains("record link target") {
				Some("undefined-record-link")
			} else {
				None
			};

			if let Some(code) = suppress_code {
				let line_idx = diag.range.start.line as usize;
				let line = source.lines().nth(line_idx).unwrap_or("");

				// Find end of actual code (before any existing comment)
				let code_end = line
					.find("--")
					.or_else(|| line.find("//"))
					.unwrap_or(line.len());
				let trimmed_end = line[..code_end].trim_end().len();

				let insert_pos = Position {
					line: diag.range.start.line,
					character: trimmed_end as u32,
				};

				let edit = TextEdit {
					range: Range {
						start: insert_pos,
						end: insert_pos,
					},
					new_text: format!("  -- surql-allow: {code}"),
				};

				let mut changes = std::collections::HashMap::new();
				changes.insert(uri.clone(), vec![edit]);

				actions.push(CodeActionOrCommand::CodeAction(CodeAction {
					title: format!("Suppress with -- surql-allow: {code}"),
					kind: Some(CodeActionKind::QUICKFIX),
					diagnostics: Some(vec![diag.clone()]),
					edit: Some(WorkspaceEdit {
						changes: Some(changes),
						..Default::default()
					}),
					..Default::default()
				}));
			}
		}

		if actions.is_empty() {
			Ok(None)
		} else {
			Ok(Some(actions))
		}
	}

	async fn did_close(&self, params: DidCloseTextDocumentParams) {
		let uri = params.text_document.uri;
		self.documents.close(&uri);
		self.document_schemas.remove(&uri);
		self.client.publish_diagnostics(uri, vec![], None).await;
	}
}

pub(crate) fn find_record_link_range(source: &str, table_name: &str) -> Option<Range> {
	let search = format!("record<{table_name}>");
	let search_upper = format!("record<{}>", table_name.to_uppercase());
	for (line_num, line) in source.lines().enumerate() {
		let upper_line = line.to_uppercase();
		for pattern in [&search, &search_upper] {
			let upper_pattern = pattern.to_uppercase();
			if let Some(col) = upper_line.find(&upper_pattern) {
				let inner_start = col + "record<".len();
				return Some(Range {
					start: Position {
						line: line_num as u32,
						character: inner_start as u32,
					},
					end: Position {
						line: line_num as u32,
						character: (inner_start + table_name.len()) as u32,
					},
				});
			}
		}
	}
	None
}

/// Check whether a source line contains a suppress comment for the given diagnostic code.
///
/// Supports both `// surql-allow: <code>` and `-- surql-allow: <code>` comment styles.
/// The code is matched case-insensitively.
pub(crate) fn line_has_suppress(source: &str, line_number: u32, code: &str) -> bool {
	let Some(line_text) = source.lines().nth(line_number as usize) else {
		return false;
	};
	let lower = line_text.to_lowercase();
	let code_lower = code.to_lowercase();
	for comment_marker in ["//", "--"] {
		if let Some(pos) = lower.find(comment_marker) {
			let after_marker = &lower[pos + comment_marker.len()..];
			let trimmed = after_marker.trim_start();
			if let Some(rest) = trimmed.strip_prefix("surql-allow:") {
				let allowed = rest.trim();
				if allowed == code_lower
					|| allowed.starts_with(&format!("{code_lower} "))
					|| allowed.starts_with(&format!("{code_lower},"))
				{
					return true;
				}
			}
		}
	}
	false
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

/// Convert a byte offset in source text to an LSP Position (0-indexed line, UTF-16 column).
pub(crate) fn byte_offset_to_position(source: &str, offset: usize) -> Position {
	let offset = offset.min(source.len());
	let before = &source[..offset];
	let line = before.matches('\n').count() as u32;
	let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
	let line_text = &source[line_start..offset];
	let character = line_text.chars().map(|c| c.len_utf16() as u32).sum();
	Position { line, character }
}

/// Extract the word (identifier) at a cursor position, with its range.
///
/// Converts the UTF-16 `position.character` to a byte offset before scanning,
/// and converts byte offsets back to UTF-16 code units for the returned range.
pub(crate) fn word_and_range_at_position(
	source: &str,
	position: Position,
) -> (String, Option<Range>) {
	let line = match source.split('\n').nth(position.line as usize) {
		Some(l) => l.strip_suffix('\r').unwrap_or(l),
		None => return (String::new(), None),
	};

	// Convert UTF-16 character offset to byte offset within the line
	let mut utf16_count = 0u32;
	let mut col = line.len(); // default: end of line
	for (byte_idx, ch) in line.char_indices() {
		if utf16_count >= position.character {
			col = byte_idx;
			break;
		}
		utf16_count += ch.len_utf16() as u32;
	}

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

		// Convert byte offsets back to UTF-16 code units for the range
		let start_utf16: u32 = line[..start].chars().map(|c| c.len_utf16() as u32).sum();
		let end_utf16: u32 = line[..end].chars().map(|c| c.len_utf16() as u32).sum();

		let range = Range {
			start: Position {
				line: position.line,
				character: start_utf16,
			},
			end: Position {
				line: position.line,
				character: end_utf16,
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
		Err(e) => {
			tracing::error!("Lexer panicked during semantic token extraction: {e:?}");
			return;
		}
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

/// Detect compound keywords like "DEFINE TABLE", "DEFINE FIELD", "ORDER BY", etc.
fn detect_compound_keyword(source: &str, position: Position, current_word: &str) -> Option<String> {
	let line = source.split('\n').nth(position.line as usize)?;
	let col = position.character as usize;
	let before = if col < line.len() { &line[..col] } else { line };
	let trimmed = before.trim_end();
	let prev_end = trimmed.rfind(|c: char| !c.is_ascii_whitespace())?;
	let prev_word_end = prev_end + 1;
	let prev_start = trimmed[..prev_word_end]
		.rfind(|c: char| c.is_ascii_whitespace())
		.map(|i| i + 1)
		.unwrap_or(0);
	let prev_word = trimmed[prev_start..prev_word_end].to_uppercase();
	let word_upper = current_word.to_uppercase();
	let compound = format!("{prev_word} {word_upper}");
	match compound.as_str() {
		"DEFINE TABLE" | "DEFINE FIELD" | "DEFINE INDEX" | "DEFINE FUNCTION" | "DEFINE EVENT"
		| "DEFINE ANALYZER" | "DEFINE PARAM" | "DEFINE ACCESS" | "DEFINE NAMESPACE"
		| "DEFINE DATABASE" | "ORDER BY" | "GROUP BY" | "SPLIT ON" | "INSERT INTO"
		| "TYPE NORMAL" | "TYPE RELATION" | "TYPE ANY" => Some(compound),
		_ => None,
	}
}
