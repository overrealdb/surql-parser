//! surql-transform: AST-level Rust source transformer.
//!
//! Transforms SurrealDB parser source code into standalone surql-parser crate
//! by rewriting imports, removing execution-only code, and wrapping attributes.
//!
//! Usage:
//!   surql-transform --mappings mappings.toml --input src/upstream/ [--dry-run]

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser as ClapParser;
use proc_macro2::Span;
use syn::visit_mut::VisitMut;
use syn::{
	Attribute, File, Ident, Item, ItemImpl, ItemMod, Meta, Path as SynPath, PathSegment,
	punctuated::Punctuated,
};
use walkdir::WalkDir;

mod mappings;
use mappings::Mappings;

#[derive(ClapParser, Debug)]
#[command(
	name = "surql-transform",
	about = "Transform SurrealDB source into standalone parser"
)]
struct Cli {
	/// Path to mappings.toml
	#[arg(short, long, default_value = "mappings.toml")]
	mappings: PathBuf,

	/// Input directory (src/upstream/)
	#[arg(short, long)]
	input: PathBuf,

	/// Dry run — report what would change without writing
	#[arg(long, default_value_t = false)]
	dry_run: bool,

	/// Strict mode — fail on unknown crate:: imports
	#[arg(long, default_value_t = false)]
	strict: bool,
}

/// Collected statistics from transformation
#[derive(Default)]
struct Stats {
	files_processed: usize,
	imports_rewritten: usize,
	impls_removed: usize,
	modules_removed: usize,
	attrs_wrapped: usize,
	unknown_imports: Vec<(PathBuf, String)>,
}

/// The main AST transformer
struct Transformer {
	mappings: Mappings,
	stats: Stats,
	current_file: PathBuf,
}

impl Transformer {
	fn new(mappings: Mappings) -> Self {
		Self {
			mappings,
			stats: Stats::default(),
			current_file: PathBuf::new(),
		}
	}

	/// Check if a syn::Path starts with segments matching a given prefix string like "crate::sql"
	fn path_matches_prefix(path: &SynPath, prefix: &str) -> bool {
		let prefix_segments: Vec<&str> = prefix.split("::").collect();
		if path.segments.len() < prefix_segments.len() {
			return false;
		}
		path.segments
			.iter()
			.zip(prefix_segments.iter())
			.all(|(seg, &expected)| seg.ident == expected)
	}

	/// Rewrite a syn::Path by replacing prefix segments
	fn rewrite_path(path: &mut SynPath, old_prefix: &str, new_prefix: &str) {
		let old_segments: Vec<&str> = old_prefix.split("::").collect();
		let new_segments: Vec<&str> = new_prefix.split("::").collect();

		if path.segments.len() < old_segments.len() {
			return;
		}

		// Build new segments: replacement prefix + remaining original segments
		let remaining: Vec<PathSegment> = path
			.segments
			.iter()
			.skip(old_segments.len())
			.cloned()
			.collect();

		let mut new_punctuated = Punctuated::new();
		for seg_str in &new_segments {
			new_punctuated.push(PathSegment {
				ident: Ident::new(seg_str, Span::call_site()),
				arguments: syn::PathArguments::None,
			});
		}
		for seg in remaining {
			new_punctuated.push(seg);
		}

		path.segments = new_punctuated;
	}

	/// Check if a stringified type/path references a stripped module
	fn references_stripped_module(&self, s: &str) -> bool {
		self.mappings
			.remove
			.strip_modules
			.iter()
			.any(|prefix| s.contains(&prefix.replace("::", " :: ")))
	}

	/// Check if a use item imports from a stripped module
	fn is_stripped_use(&self, tree: &syn::ItemUse) -> bool {
		let use_str = quote::quote!(#tree).to_string();
		self.mappings
			.remove
			.strip_modules
			.iter()
			.any(|prefix| use_str.contains(&prefix.replace("::", " :: ")))
	}

	/// Check if an impl references stripped modules (in self_ty or trait)
	fn is_stripped_impl(&self, item: &ItemImpl) -> bool {
		let self_type = quote::quote!(#item.self_ty).to_string();
		if self.references_stripped_module(&self_type) {
			return true;
		}
		if let Some((_, ref trait_path, _)) = item.trait_ {
			let trait_str = quote::quote!(#trait_path).to_string();
			if self.references_stripped_module(&trait_str) {
				return true;
			}
		}
		false
	}

	/// Check if an impl block contains execution-only methods
	fn is_execution_impl(&self, item: &ItemImpl) -> bool {
		for impl_item in &item.items {
			if let syn::ImplItem::Fn(method) = impl_item {
				for input in &method.sig.inputs {
					if let syn::FnArg::Typed(pat_type) = input {
						let type_str = quote::quote!(#pat_type).to_string();
						for param in &self.mappings.remove.execution_params {
							if type_str.contains(param) {
								return true;
							}
						}
					}
				}
			}
		}
		false
	}

	/// Check if an impl is a cross-layer From impl (e.g., From<sql::X> for expr::X)
	fn is_cross_layer_from(&self, item: &ItemImpl) -> bool {
		// Check if this is a From<...> impl
		if let Some((_, ref trait_path, _)) = item.trait_ {
			let trait_str = quote::quote!(#trait_path).to_string();
			if trait_str.contains("From") {
				let self_type = quote::quote!(#item.self_ty).to_string();
				for [from_mod, to_mod] in &self.mappings.remove.cross_layer_from_impls {
					// Check if From<from_mod::X> for to_mod::Y
					if (trait_str.contains(from_mod) && self_type.contains(to_mod))
						|| (trait_str.contains(to_mod) && self_type.contains(from_mod))
					{
						return true;
					}
				}
			}
		}
		false
	}

	/// Check if an attribute matches a name we want to cfg-wrap
	#[allow(dead_code)]
	fn should_cfg_wrap(attr: &Attribute, cfg_wrap: &HashMap<String, String>) -> Option<String> {
		if let Meta::List(meta_list) = &attr.meta {
			let path_str = quote::quote!(#meta_list.path).to_string();
			for (attr_name, feature) in cfg_wrap {
				if path_str.contains(attr_name) {
					return Some(feature.clone());
				}
			}
		}
		None
	}

	/// Check if an item has #[cfg(test)] attribute
	fn has_cfg_test(attrs: &[Attribute]) -> bool {
		attrs.iter().any(|attr| {
			if let Meta::List(meta_list) = &attr.meta {
				let path_str = quote::quote!(#meta_list.path).to_string();
				if path_str.contains("cfg") {
					let tokens = meta_list.tokens.to_string();
					return tokens.contains("test");
				}
			}
			false
		})
	}

	/// Collect unknown crate:: imports for warning
	fn check_unknown_import(&mut self, path: &SynPath) {
		if path.segments.first().is_some_and(|s| s.ident == "crate") && path.segments.len() >= 2 {
			let path_str = quote::quote!(#path).to_string();

			// Skip if it matches a known rewrite
			let known_rewrite = self
				.mappings
				.import_rewrites
				.keys()
				.any(|prefix| Self::path_matches_prefix(path, prefix));

			// Skip if it matches a stripped module
			let known_stripped = self.references_stripped_module(&path_str);

			if !known_rewrite && !known_stripped {
				self.stats
					.unknown_imports
					.push((self.current_file.clone(), path_str));
			}
		}
	}
}

impl VisitMut for Transformer {
	fn visit_file_mut(&mut self, file: &mut File) {
		// Single pass: remove all unwanted items
		file.items.retain(|item| {
			match item {
				// Remove use imports from stripped modules
				Item::Use(u) => {
					if self.is_stripped_use(u) {
						self.stats.imports_rewritten += 1;
						return false;
					}
					true
				}
				// Remove #[cfg(test)] modules + named test modules
				Item::Mod(m) => {
					if Self::has_cfg_test(&m.attrs) {
						self.stats.modules_removed += 1;
						return false;
					}
					if self
						.mappings
						.remove
						.remove_modules
						.contains(&m.ident.to_string())
					{
						self.stats.modules_removed += 1;
						return false;
					}
					true
				}
				// Remove execution impls, cross-layer From, and stripped-module impls
				Item::Impl(impl_item) => {
					if self.is_stripped_impl(impl_item) {
						self.stats.impls_removed += 1;
						return false;
					}
					if self.is_execution_impl(impl_item) {
						self.stats.impls_removed += 1;
						return false;
					}
					if self.is_cross_layer_from(impl_item) {
						self.stats.impls_removed += 1;
						return false;
					}
					true
				}
				// Remove #[cfg(test)] functions
				Item::Fn(f) => {
					if Self::has_cfg_test(&f.attrs) {
						return false;
					}
					true
				}
				_ => true,
			}
		});

		// Continue visiting remaining items (rewrites paths, wraps attrs, etc.)
		syn::visit_mut::visit_file_mut(self, file);
	}

	fn visit_path_mut(&mut self, path: &mut SynPath) {
		// Check for unknown imports first
		self.check_unknown_import(path);

		// Apply import rewrites (longest prefix match first)
		let mut best_match: Option<(&str, &str)> = None;
		for (old_prefix, new_prefix) in &self.mappings.import_rewrites {
			if Self::path_matches_prefix(path, old_prefix)
				&& (best_match.is_none() || old_prefix.len() > best_match.unwrap().0.len())
			{
				best_match = Some((old_prefix.as_str(), new_prefix.as_str()));
			}
		}

		if let Some((old, new)) = best_match {
			Self::rewrite_path(path, old, new);
			self.stats.imports_rewritten += 1;
		}

		syn::visit_mut::visit_path_mut(self, path);
	}

	fn visit_item_fn_mut(&mut self, func: &mut syn::ItemFn) {
		// Remove attributes listed in remove_attrs (e.g., #[instrument(...)])
		let remove_attrs = &self.mappings.attributes.remove_attrs;
		let original_count = func.attrs.len();
		func.attrs.retain(|attr| {
			let attr_str = quote::quote!(#attr).to_string();
			!remove_attrs.iter().any(|name| attr_str.contains(name))
		});
		self.stats.attrs_wrapped += original_count - func.attrs.len();

		syn::visit_mut::visit_item_fn_mut(self, func);
	}

	fn visit_item_mod_mut(&mut self, module: &mut ItemMod) {
		// Recurse into inline modules
		if let Some((_, ref mut items)) = module.content {
			// Remove #[cfg(test)] items inside modules
			items.retain(|item| {
				let is_cfg_test = match item {
					Item::Mod(m) => Self::has_cfg_test(&m.attrs),
					Item::Fn(f) => Self::has_cfg_test(&f.attrs),
					_ => false,
				};
				if is_cfg_test {
					self.stats.modules_removed += 1;
				}
				!is_cfg_test
			});
		}
		syn::visit_mut::visit_item_mod_mut(self, module);
	}
}

fn transform_file(transformer: &mut Transformer, path: &Path) -> Result<String> {
	let source = std::fs::read_to_string(path)
		.with_context(|| format!("Failed to read {}", path.display()))?;

	// Phase 1: Text-level import rewrites BEFORE parsing.
	// syn's UseTree doesn't go through visit_path_mut, so we rewrite
	// `crate::X` prefixes at text level first. This handles use statements,
	// type annotations, and all other path occurrences reliably.
	let mut source = source;

	// Sort rewrites by length descending so longer prefixes match first
	// (e.g., "crate::dbs::Capabilities" before "crate::dbs")
	let mut rewrites: Vec<(&str, &str)> = transformer
		.mappings
		.import_rewrites
		.iter()
		.map(|(k, v)| (k.as_str(), v.as_str()))
		.collect();
	rewrites.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

	for (old_prefix, new_prefix) in &rewrites {
		source = source.replace(old_prefix, new_prefix);
	}

	// Also strip `use crate::expr::*` and similar lines for stripped modules
	for stripped in &transformer.mappings.remove.strip_modules {
		// Remove entire use lines that reference stripped modules
		let lines: Vec<&str> = source.lines().collect();
		let mut filtered = Vec::with_capacity(lines.len());
		let mut in_stripped_use = false;
		for line in lines {
			if line.trim_start().starts_with("use ") && line.contains(stripped) {
				transformer.stats.imports_rewritten += 1;
				// Multi-line use: skip until semicolon
				if !line.contains(';') {
					in_stripped_use = true;
				}
				continue;
			}
			if in_stripped_use {
				if line.contains(';') {
					in_stripped_use = false;
				}
				continue;
			}
			filtered.push(line);
		}
		source = filtered.join("\n");
	}

	// Phase 2: Parse and apply AST-level transformations
	let mut ast: File =
		syn::parse_file(&source).with_context(|| format!("Failed to parse {}", path.display()))?;

	transformer.current_file = path.to_path_buf();
	transformer.visit_file_mut(&mut ast);
	transformer.stats.files_processed += 1;

	let output = prettyplease::unparse(&ast);
	Ok(output)
}

fn main() -> Result<()> {
	let cli = Cli::parse();

	let mappings = Mappings::load(&cli.mappings)
		.with_context(|| format!("Failed to load mappings from {}", cli.mappings.display()))?;

	let mut transformer = Transformer::new(mappings);

	let rs_files: Vec<PathBuf> = WalkDir::new(&cli.input)
		.into_iter()
		.filter_map(|e| e.ok())
		.filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
		.map(|e| e.into_path())
		.collect();

	eprintln!(
		"Processing {} Rust files in {}",
		rs_files.len(),
		cli.input.display()
	);

	for path in &rs_files {
		match transform_file(&mut transformer, path) {
			Ok(output) => {
				if cli.dry_run {
					let rel = path.strip_prefix(&cli.input).unwrap_or(path);
					eprintln!("  [dry-run] Would transform: {}", rel.display());
				} else {
					std::fs::write(path, output)
						.with_context(|| format!("Failed to write {}", path.display()))?;
				}
			}
			Err(e) => {
				eprintln!("  [ERROR] {}: {e:#}", path.display());
				// Continue processing other files
			}
		}
	}

	// Report
	let stats = &transformer.stats;
	eprintln!();
	eprintln!("=== Transform Summary ===");
	eprintln!("  Files processed:    {}", stats.files_processed);
	eprintln!("  Imports rewritten:  {}", stats.imports_rewritten);
	eprintln!("  Impl blocks removed: {}", stats.impls_removed);
	eprintln!("  Modules removed:    {}", stats.modules_removed);
	eprintln!("  Attributes wrapped: {}", stats.attrs_wrapped);

	if !stats.unknown_imports.is_empty() {
		eprintln!();
		eprintln!(
			"=== Unknown crate:: imports ({}) ===",
			stats.unknown_imports.len()
		);
		let mut seen = HashSet::new();
		for (file, import) in &stats.unknown_imports {
			let key = import.clone();
			if seen.insert(key) {
				let rel = file.strip_prefix(&cli.input).unwrap_or(file);
				eprintln!("  WARNING: {} in {}", import, rel.display());
			}
		}
		eprintln!();
		eprintln!("Add mappings to mappings.toml or create a patch to fix these.");

		if cli.strict || transformer.mappings.warnings.fail_on_unknown {
			bail!(
				"Strict mode: {} unknown imports found. Fix them before proceeding.",
				stats.unknown_imports.len()
			);
		}
	}

	Ok(())
}
