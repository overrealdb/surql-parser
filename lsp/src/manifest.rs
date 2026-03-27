use surql_parser::SchemaGraph;

/// Write file manifest and schema cache for the Zed extension.
///
/// Writes to two locations:
/// 1. `surql-lsp-out/files.json` in project root (for general use)
/// 2. Zed extension work dir (WASM can read via `std::fs` from `.`)
pub(crate) fn write_file_manifest(root: &std::path::Path, schema: &SchemaGraph) {
	let mut abs_files = Vec::new();
	surql_parser::collect_surql_files(root, &mut abs_files);
	let mut files: Vec<String> = abs_files
		.iter()
		.filter_map(|p| p.strip_prefix(root).ok())
		.map(|rel| rel.to_string_lossy().to_string())
		.collect();
	files.sort();

	// Build schema cache from files
	let mut schema_text = String::new();
	let mut file_count = 0;
	for rel_path in &files {
		let full_path = root.join(rel_path);
		let content = match std::fs::read_to_string(&full_path) {
			Ok(c) => c,
			Err(e) => {
				tracing::warn!("Skipping {}: {e}", full_path.display());
				continue;
			}
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

	// Build relations graph and info summary from SchemaGraph
	let relations_text = build_relations_graph(schema);
	let check_text = build_check_results(root, &files, schema);
	let migrations_text = build_migration_status(root);
	let info_text = build_info_summary(schema, &files, root);
	let docs_text = schema.build_docs_markdown();
	let table_count = schema.table_names().count();
	let graph_text = if table_count <= 100 {
		schema.build_graph_tree_markdown()
	} else {
		format!(
			"# Schema Graph\n\n\
			 Schema has {table_count} tables (>100). \
			 Use `surql graph` CLI for graph output.\n"
		)
	};

	// Write to project root
	let manifest_dir = root.join("surql-lsp-out");
	if let Err(e) = std::fs::create_dir_all(&manifest_dir) {
		tracing::warn!("Cannot create surql-lsp-out dir: {e}");
	} else {
		let files_json = serde_json::to_string_pretty(&files).unwrap_or_else(|_| "[]".to_string());
		for (name, content) in [
			("files.json", files_json.as_str()),
			("schema.md", schema_text.as_str()),
			("relations.md", relations_text.as_str()),
			("info.md", info_text.as_str()),
			("diagnostics.md", check_text.as_str()),
			("migrations.md", migrations_text.as_str()),
			("docs.md", docs_text.as_str()),
			("graph.md", graph_text.as_str()),
		] {
			if let Err(e) = std::fs::write(manifest_dir.join(name), content) {
				tracing::warn!("Failed to write {name}: {e}");
			}
		}
	}

	// Write to Zed extension work dir (WASM reads from ".")
	if let Ok(home) = std::env::var("HOME") {
		let candidates = [
			std::path::PathBuf::from(&home)
				.join("Library/Application Support/Zed/extensions/work/surrealql"),
			std::path::PathBuf::from(&home).join(".local/share/zed/extensions/work/surrealql"),
		];
		for zed_ext_dir in &candidates {
			if zed_ext_dir.exists() {
				for (name, content) in [
					("schema.md", &schema_text),
					("relations.md", &relations_text),
					("info.md", &info_text),
					("diagnostics.md", &check_text),
					("migrations.md", &migrations_text),
					("docs.md", &docs_text),
					("graph.md", &graph_text),
				] {
					if let Err(e) = std::fs::write(zed_ext_dir.join(name), content) {
						tracing::warn!("Failed to write {name} to Zed ext dir: {e}");
					}
				}
				tracing::info!(
					"Schema + relations + info + check + migrations + docs + graph cache written to Zed extension dir"
				);
				break;
			}
		}
	}
}

pub(crate) fn build_info_summary(
	schema: &SchemaGraph,
	files: &[String],
	root: &std::path::Path,
) -> String {
	let table_count = schema.table_names().count();
	let fn_count = schema.function_names().count();
	let param_count = schema.param_names().count();

	let mut total_fields = 0;
	let mut total_indexes = 0;
	let mut total_events = 0;
	let mut total_relations = 0;
	let mut schemafull_count = 0;

	for table_name in schema.table_names() {
		if let Some(table) = schema.table(table_name) {
			total_fields += table.fields.len();
			total_indexes += table.indexes.len();
			total_events += table.events.len();
			if table.full {
				schemafull_count += 1;
			}
			for field in &table.fields {
				total_relations += field.record_links.len();
			}
		}
	}

	let manifest_info = detect_overshift_manifest(root)
		.map(|(ns, db)| format!("- **Namespace:** `{ns}`\n- **Database:** `{db}`\n"))
		.unwrap_or_default();

	format!(
		"# SurrealQL Workspace Info\n\n\
		 ## Overview\n\n\
		 - **{table_count}** tables ({schemafull_count} SCHEMAFULL, {} SCHEMALESS)\n\
		 - **{total_fields}** fields\n\
		 - **{total_indexes}** indexes\n\
		 - **{total_events}** events\n\
		 - **{total_relations}** record links\n\
		 - **{fn_count}** functions\n\
		 - **{param_count}** params\n\
		 - **{}** .surql files\n\n\
		 {manifest_info}\
		 ## Available Commands\n\n\
		 | Command | Description |\n\
		 |---------|-------------|\n\
		 | `/surql-schema` | Show all DEFINE statements |\n\
		 | `/surql-relations` | Table relationship graph |\n\
		 | `/surql-check` | Validation results for all .surql files |\n\
		 | `/surql-migrations` | Overshift migration status |\n\
		 | `/surql-docs` | Schema documentation from COMMENT fields |\n\
		 | `/surql-graph` | Schema dependency tree (record links) |\n\
		 | `/surql-dependents` | Reverse dependencies for a table |\n\
		 | `/surql-info` | This summary |\n",
		table_count - schemafull_count,
		files.len()
	)
}

pub(crate) fn build_relations_graph(schema: &SchemaGraph) -> String {
	let mut table_names: Vec<&str> = schema.table_names().collect();
	table_names.sort();

	if table_names.is_empty() {
		return "No tables defined".to_string();
	}

	let mut text = String::new();
	let mut edges: Vec<(String, String, String)> = Vec::new();

	for table_name in &table_names {
		if let Some(table) = schema.table(table_name) {
			for field in &table.fields {
				for link in &field.record_links {
					edges.push((table_name.to_string(), link.clone(), field.name.clone()));
				}
			}
		}
	}

	text.push_str(&format!("**{} tables**\n\n", table_names.len()));

	text.push_str("```\n");
	for table_name in &table_names {
		let table = match schema.table(table_name) {
			Some(t) => t,
			None => continue,
		};

		let schema_type = if table.full {
			"SCHEMAFULL"
		} else {
			"SCHEMALESS"
		};
		let field_count = table.fields.len();
		let index_count = table.indexes.len();
		let event_count = table.events.len();

		let mut meta = vec![schema_type.to_string()];
		if field_count > 0 {
			meta.push(format!("{field_count}f"));
		}
		if index_count > 0 {
			meta.push(format!("{index_count}i"));
		}
		if event_count > 0 {
			meta.push(format!("{event_count}e"));
		}

		text.push_str(&format!("[{table_name}] ({})\n", meta.join(", ")));

		let outgoing: Vec<_> = edges
			.iter()
			.filter(|(from, _, _)| from == *table_name)
			.collect();
		for (_, to, field) in &outgoing {
			text.push_str(&format!(
				"  \u{2514}\u{2500}\u{2500} .{field} \u{2192} [{to}]\n"
			));
		}

		let incoming: Vec<_> = edges
			.iter()
			.filter(|(_, to, _)| to == *table_name)
			.collect();
		for (from, _, field) in &incoming {
			text.push_str(&format!(
				"  \u{2514}\u{2500}\u{2500} [{from}].{field} \u{2192} *\n"
			));
		}
	}
	text.push_str("```\n");

	if !edges.is_empty() {
		text.push_str(&format!("\n**{} relation(s)**\n\n", edges.len()));
		for (from, to, field) in &edges {
			text.push_str(&format!("- `{from}.{field}` \u{2192} `{to}`\n"));
		}
	}

	text
}

pub(crate) fn build_check_results(
	root: &std::path::Path,
	files: &[String],
	schema: &SchemaGraph,
) -> String {
	let mut errors: Vec<String> = Vec::new();
	let mut warnings: Vec<String> = Vec::new();
	let mut checked = 0;

	for rel_path in files {
		let full_path = root.join(rel_path);
		let source = match std::fs::read_to_string(&full_path) {
			Ok(c) => c,
			Err(_) => continue,
		};
		checked += 1;

		let result = crate::diagnostics::compute_with_recovery(&source);
		for d in &result.diagnostics {
			let line = d.range.start.line + 1;
			let col = d.range.start.character + 1;
			let msg = &d.message;
			let severity = d
				.severity
				.unwrap_or(tower_lsp::lsp_types::DiagnosticSeverity::ERROR);
			if severity == tower_lsp::lsp_types::DiagnosticSeverity::ERROR {
				errors.push(format!("- {rel_path}:{line}:{col} \u{2014} {msg}"));
			} else {
				warnings.push(format!("- {rel_path}:{line}:{col} \u{2014} {msg}"));
			}
		}

		if schema.table_names().count() > 0 {
			for table_ref in crate::context::extract_table_references(&source) {
				if schema.table(&table_ref.name).is_none() {
					let line = table_ref.line + 1;
					warnings.push(format!(
						"- {rel_path}:{line} \u{2014} Table '{}' not defined in workspace",
						table_ref.name
					));
				}
			}
		}
	}

	let mut text = String::from("# SurrealQL Check Results\n\n");

	if !errors.is_empty() {
		text.push_str(&format!("## Errors ({})\n\n", errors.len()));
		for e in &errors {
			text.push_str(e);
			text.push('\n');
		}
		text.push('\n');
	}

	if !warnings.is_empty() {
		text.push_str(&format!("## Warnings ({})\n\n", warnings.len()));
		for w in &warnings {
			text.push_str(w);
			text.push('\n');
		}
		text.push('\n');
	}

	if errors.is_empty() && warnings.is_empty() {
		text.push_str("No errors or warnings found.\n\n");
	}

	text.push_str(&format!(
		"## Summary\n\n{checked} files checked, {} errors, {} warnings\n",
		errors.len(),
		warnings.len()
	));

	text
}

pub(crate) fn build_migration_status(root: &std::path::Path) -> String {
	let manifest_candidates = [
		root.join("manifest.toml"),
		root.join("surql/manifest.toml"),
		root.join("sql/manifest.toml"),
	];

	let (manifest_path, manifest_content) = match manifest_candidates
		.iter()
		.find_map(|p| std::fs::read_to_string(p).ok().map(|c| (p.clone(), c)))
	{
		Some(pair) => pair,
		None => {
			return "# Overshift Migrations\n\n\
				No `manifest.toml` found in workspace.\n"
				.to_string();
		}
	};

	let manifest: toml::Value = match toml::from_str(&manifest_content) {
		Ok(v) => v,
		Err(e) => {
			return format!(
				"# Overshift Migrations\n\n\
				 Failed to parse `{}`: {e}\n",
				manifest_path.display()
			);
		}
	};

	let mut text = String::from("# Overshift Migrations\n\n");

	if let Some(meta) = manifest.get("meta") {
		let ns = meta.get("ns").and_then(|v| v.as_str()).unwrap_or("(unset)");
		let db = meta.get("db").and_then(|v| v.as_str()).unwrap_or("(unset)");
		text.push_str(&format!("**NS:** {ns} | **DB:** {db}\n\n"));
	}

	if let Some(modules) = manifest.get("modules").and_then(|v| v.as_array())
		&& !modules.is_empty()
	{
		text.push_str(&format!("## Schema Modules ({})\n\n", modules.len()));
		for m in modules {
			let name = m.get("name").and_then(|v| v.as_str()).unwrap_or("?");
			let deps = m
				.get("depends_on")
				.and_then(|v| v.as_array())
				.map(|arr| {
					arr.iter()
						.filter_map(|d| d.as_str())
						.collect::<Vec<_>>()
						.join(", ")
				})
				.unwrap_or_default();
			if deps.is_empty() {
				text.push_str(&format!("- {name} (no deps)\n"));
			} else {
				text.push_str(&format!("- {name} (depends: {deps})\n"));
			}
		}
		text.push('\n');
	}

	let migrations_dir = manifest_path.parent().map(|p| p.join("migrations"));
	let mut migration_entries: Vec<(u32, String, String)> = Vec::new();

	if let Some(ref dir) = migrations_dir
		&& dir.exists()
		&& let Ok(entries) = std::fs::read_dir(dir)
	{
		let mut sorted: Vec<_> = entries.filter_map(|e| e.ok()).collect();
		sorted.sort_by_key(|e| e.file_name());
		for entry in sorted {
			let path = entry.path();
			if path.extension().is_some_and(|ext| ext == "surql")
				&& let Some(stem) = path.file_stem().and_then(|s| s.to_str())
				&& let Some((version, name)) = parse_migration_stem(stem)
			{
				let content = match std::fs::read_to_string(&path) {
					Ok(c) => c,
					Err(e) => {
						tracing::warn!("Cannot read migration {}: {e}", path.display());
						continue;
					}
				};
				let checksum = compute_short_checksum(&content);
				migration_entries.push((version, name, checksum));
			}
		}
	}

	if !migration_entries.is_empty() {
		text.push_str(&format!("## Migrations ({})\n\n", migration_entries.len()));
		for (version, name, checksum) in &migration_entries {
			text.push_str(&format!("- v{version:03}_{name} \u{2014} {checksum}\n"));
		}
		text.push('\n');
	} else {
		text.push_str("## Migrations\n\nNo migration files found.\n");
	}

	text
}

fn parse_migration_stem(stem: &str) -> Option<(u32, String)> {
	let stripped = stem.strip_prefix('v')?;
	let underscore_pos = stripped.find('_')?;
	let version: u32 = stripped[..underscore_pos].parse().ok()?;
	let name = stripped[underscore_pos + 1..].to_string();
	Some((version, name))
}

fn compute_short_checksum(content: &str) -> String {
	// FNV-1a 64-bit — deterministic across Rust versions and platforms
	const FNV_OFFSET: u64 = 0xcbf29ce484222325;
	const FNV_PRIME: u64 = 0x00000100000001B3;
	let mut hash = FNV_OFFSET;
	for byte in content.as_bytes() {
		hash ^= *byte as u64;
		hash = hash.wrapping_mul(FNV_PRIME);
	}
	format!("{:08x}", hash & 0xFFFF_FFFF)
}

/// Detect overshift manifest.toml in the workspace.
/// Looks for manifest.toml in common locations: root, surql/, sql/.
pub(crate) fn detect_overshift_manifest(root: &std::path::Path) -> Option<(String, String)> {
	let candidates = [
		root.join("manifest.toml"),
		root.join("surql/manifest.toml"),
		root.join("sql/manifest.toml"),
	];
	for path in &candidates {
		if let Ok(content) = std::fs::read_to_string(path)
			&& let Ok(manifest) = toml::from_str::<toml::Value>(&content)
			&& let Some(meta) = manifest.get("meta")
		{
			let ns = meta.get("ns").and_then(|v| v.as_str())?.to_string();
			let db = meta.get("db").and_then(|v| v.as_str())?.to_string();
			tracing::info!(
				"Detected overshift manifest at {}: NS={ns}, DB={db}",
				path.display()
			);
			return Some((ns, db));
		}
	}
	None
}
