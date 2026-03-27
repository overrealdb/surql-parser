//! Compile-time SurrealQL validation macros.
//!
//! Provides proc macros that validate SurrealQL at compile time using `surql-parser`.
//!
//! # Macros
//!
//! - [`surql_check!`] — validate a SurrealQL string at compile time
//! - [`surql_query!`] — validate SurrealQL + verify parameter bindings at compile time
//! - [`#[surql_function]`](macro@surql_function) — validate a SurrealQL function name at compile time

use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr, Token, parse_macro_input};

mod query_inference;
mod type_check;

use query_inference::infer_param_types_from_query;
use type_check::{
	extract_rust_param_types, format_rust_type, surql_type_matches_rust, type_hint_for_surql,
};

// ─── Macros ───

/// Validates a SurrealQL string at compile time.
///
/// Parses the given string literal as SurrealQL. If parsing succeeds, the macro
/// expands to the original string literal (`&'static str`). If parsing fails,
/// a compile error is emitted with the parse error message.
///
/// # Example
///
/// ```
/// use surql_macros::surql_check;
///
/// let query = surql_check!("SELECT * FROM user WHERE age > 18");
/// assert_eq!(query, "SELECT * FROM user WHERE age > 18");
/// ```
///
/// ```compile_fail
/// use surql_macros::surql_check;
///
/// // This will not compile — invalid SurrealQL:
/// let query = surql_check!("SELEC * FORM user");
/// ```
#[proc_macro]
pub fn surql_check(input: TokenStream) -> TokenStream {
	let lit = parse_macro_input!(input as LitStr);
	let sql = lit.value();
	match surql_parser::parse(&sql) {
		Ok(_) => quote! { #lit }.into(),
		Err(e) => {
			let msg = format!("Invalid SurrealQL: {e}");
			syn::Error::new(lit.span(), msg).to_compile_error().into()
		}
	}
}

/// Validates a SurrealQL query and verifies parameter bindings at compile time.
///
/// This macro goes beyond [`surql_check!`] by also extracting `$param` placeholders
/// from the query and verifying that the caller provides matching parameter names.
///
/// When `schema = "path/"` is provided, the macro also performs type inference:
/// it looks up field types in the schema and checks that Rust variable types
/// are compatible with the SurrealQL field types used in WHERE clauses.
///
/// # Usage
///
/// ```
/// use surql_macros::surql_query;
///
/// // Just validate syntax (no params expected):
/// let sql = surql_query!("SELECT * FROM user");
///
/// // Validate syntax + verify params match:
/// let sql = surql_query!("SELECT * FROM user WHERE age > $min AND name = $name", min, name);
/// assert_eq!(sql, "SELECT * FROM user WHERE age > $min AND name = $name");
/// ```
///
/// ```compile_fail
/// use surql_macros::surql_query;
///
/// // Compile error: missing parameter `name`
/// let sql = surql_query!("SELECT * FROM user WHERE age > $min AND name = $name", min);
/// ```
///
/// ```compile_fail
/// use surql_macros::surql_query;
///
/// // Compile error: extra parameter `city` not found in query
/// let sql = surql_query!("SELECT * FROM user WHERE age > $min", min, city);
/// ```
#[proc_macro]
pub fn surql_query(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as SurqlQueryInput);
	let sql = input.sql.value();

	// 1. Validate SurrealQL syntax
	if let Err(e) = surql_parser::parse(&sql) {
		let msg = format!("Invalid SurrealQL: {e}");
		return syn::Error::new(input.sql.span(), msg)
			.to_compile_error()
			.into();
	}

	// 2. Extract $param names from the query
	let query_params = match surql_parser::extract_params(&sql) {
		Ok(params) => params,
		Err(e) => {
			let msg = format!("Failed to extract parameters: {e}");
			return syn::Error::new(input.sql.span(), msg)
				.to_compile_error()
				.into();
		}
	};

	// 3. If caller provided param names, verify they match
	if !input.params.is_empty() {
		let provided: Vec<String> = input.params.iter().map(|p| p.ident.to_string()).collect();

		// Check for missing params (in query but not provided)
		for qp in &query_params {
			if !provided.contains(qp) {
				let msg =
					format!("missing parameter `{qp}` — query uses ${qp} but it was not provided");
				return syn::Error::new(input.sql.span(), msg)
					.to_compile_error()
					.into();
			}
		}

		// Check for extra params (provided but not in query)
		for (i, pp) in provided.iter().enumerate() {
			if !query_params.contains(pp) {
				let msg = format!(
					"extra parameter `{pp}` — not found in query (expected: {})",
					if query_params.is_empty() {
						"none".to_string()
					} else {
						query_params
							.iter()
							.map(|p| format!("${p}"))
							.collect::<Vec<_>>()
							.join(", ")
					}
				);
				return syn::Error::new(input.params[i].ident.span(), msg)
					.to_compile_error()
					.into();
			}
		}
	}

	// 4. Schema-based type checking for parameters
	if let Some(ref schema_lit) = input.schema {
		let schema_path = schema_lit.value();
		let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
		let full_path = std::path::Path::new(&manifest_dir).join(&schema_path);

		if !full_path.exists() {
			let msg = format!("schema path '{}' does not exist", schema_path);
			return syn::Error::new(schema_lit.span(), msg)
				.to_compile_error()
				.into();
		}

		// Load all field types from the schema
		let field_types = match surql_parser::collect_field_types(&full_path) {
			Ok(fts) => fts
				.into_iter()
				.map(|ft| (ft.table, ft.field, ft.kind))
				.collect::<Vec<_>>(),
			Err(e) => {
				let msg = format!("failed to scan schema files: {e}");
				return syn::Error::new(schema_lit.span(), msg)
					.to_compile_error()
					.into();
			}
		};

		// Infer type constraints from the query
		let constraints = infer_param_types_from_query(&sql, &field_types);

		// Check typed parameters against inferred constraints
		for param in &input.params {
			if let Some(ref rust_type) = param.ty {
				let param_name = param.ident.to_string();
				let rust_type_str = format_rust_type(rust_type);

				for constraint in &constraints {
					if constraint.param_name == param_name
						&& !surql_type_matches_rust(&constraint.surql_type, &rust_type_str)
					{
						let hint = type_hint_for_surql(&constraint.surql_type);
						let msg = format!(
							"type mismatch for parameter `${param_name}`: \
							 Rust type `{rust_type_str}` is not compatible with \
							 SurrealQL type `{}`\n\
							 \x20 inferred from: {}\n\
							 \x20 expected Rust types: {hint}",
							constraint.surql_type, constraint.source
						);
						return syn::Error::new(param.ident.span(), msg)
							.to_compile_error()
							.into();
					}
				}
			}
		}
	}

	// 5. Expand to the string literal
	let lit = &input.sql;
	quote! { #lit }.into()
}

/// A parameter in surql_query! with an optional Rust type annotation.
struct QueryParam {
	ident: syn::Ident,
	ty: Option<syn::Type>,
}

/// Parsed input for `surql_query!`: a string literal, optional params with types,
/// and optional `schema = "path/"`.
struct SurqlQueryInput {
	sql: LitStr,
	params: Vec<QueryParam>,
	schema: Option<LitStr>,
}

impl syn::parse::Parse for SurqlQueryInput {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let sql: LitStr = input.parse()?;
		let mut params = Vec::new();
		let mut schema = None;

		while input.peek(Token![,]) {
			let _: Token![,] = input.parse()?;
			if input.is_empty() {
				break;
			}

			// Check for `schema = "path/"` attribute
			if input.peek(syn::Ident) {
				let fork = input.fork();
				let ident: syn::Ident = fork.parse()?;
				if ident == "schema" && fork.peek(Token![=]) {
					// Consume from the real stream
					let _ident: syn::Ident = input.parse()?;
					let _: Token![=] = input.parse()?;
					schema = Some(input.parse()?);
					continue;
				}
			}

			// Parse param: either `name` or `name: Type`
			let ident: syn::Ident = input.parse()?;
			let ty = if input.peek(Token![:]) {
				let _: Token![:] = input.parse()?;
				Some(input.parse()?)
			} else {
				None
			};

			params.push(QueryParam { ident, ty });
		}

		Ok(SurqlQueryInput {
			sql,
			params,
			schema,
		})
	}
}

/// Validates a SurrealQL function name (and optionally parameter count and types)
/// at compile time.
///
/// Place this attribute on a Rust function with a string literal argument
/// like `"fn::get_entity"`. The macro validates at compile time that:
///
/// 1. The name starts with `fn::`
/// 2. The name is syntactically valid as a SurrealQL function call
///
/// Optionally, provide `schema = "path/"` to also validate:
///
/// 3. Scans `.surql` files under the schema path for a matching DEFINE FUNCTION
/// 4. Compares Rust function parameter count with SurrealQL parameter count
/// 5. Checks that each Rust parameter type is compatible with the SurrealQL type
///
/// The annotated function is preserved as-is (the macro only adds a doc comment).
///
/// # Examples
///
/// ```
/// use surql_macros::surql_function;
///
/// #[surql_function("fn::get_entity")]
/// pub fn get_entity(name: &str) -> String {
///     format!("fn::get_entity('{name}')")
/// }
/// ```
///
/// ```compile_fail
/// use surql_macros::surql_function;
///
/// // This will not compile — missing fn:: prefix:
/// #[surql_function("get_entity")]
/// pub fn get_entity() {}
/// ```
#[proc_macro_attribute]
pub fn surql_function(attr: TokenStream, item: TokenStream) -> TokenStream {
	let parsed_attr = parse_macro_input!(attr as SurqlFunctionAttr);
	let fn_name = &parsed_attr.name;
	let name = fn_name.value();

	// Must start with fn::
	if !name.starts_with("fn::") {
		return syn::Error::new(fn_name.span(), "surql_function name must start with 'fn::'")
			.to_compile_error()
			.into();
	}

	// Validate: try parsing as a function call
	let test_call = format!("{name}()");
	if let Err(e) = surql_parser::parse(&test_call) {
		let msg = format!("Invalid SurrealQL function name '{name}': {e}");
		return syn::Error::new(fn_name.span(), msg)
			.to_compile_error()
			.into();
	}

	// Parse the Rust function to count its parameters
	let item_clone: proc_macro2::TokenStream = item.clone().into();
	let rust_fn: syn::ItemFn = match syn::parse2(item_clone) {
		Ok(f) => f,
		Err(e) => return e.to_compile_error().into(),
	};

	// If schema path provided, validate parameter count and types against DEFINE FUNCTION
	if let Some(ref schema_lit) = parsed_attr.schema {
		let schema_path = schema_lit.value();
		let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
		let full_path = std::path::Path::new(&manifest_dir).join(&schema_path);

		if !full_path.exists() {
			let msg = format!("schema path '{}' does not exist", schema_path);
			return syn::Error::new(schema_lit.span(), msg)
				.to_compile_error()
				.into();
		}

		match surql_parser::find_function_params(&name, &full_path) {
			Ok(Some(surql_params)) => {
				let rust_param_count = count_fn_params(&rust_fn);
				let surql_param_count = surql_params.len();

				if rust_param_count != surql_param_count {
					let param_list = if surql_params.is_empty() {
						"none".to_string()
					} else {
						surql_params
							.iter()
							.map(|p| format!("${}: {}", p.name, p.kind))
							.collect::<Vec<_>>()
							.join(", ")
					};
					let msg = format!(
						"parameter count mismatch for {name}: \
						 Rust function has {rust_param_count} parameter(s), \
						 but DEFINE FUNCTION has {surql_param_count} ({param_list})"
					);
					return syn::Error::new(fn_name.span(), msg)
						.to_compile_error()
						.into();
				}

				// Type checking: compare each Rust param type with SurrealQL param type
				let rust_params = extract_rust_param_types(&rust_fn);
				for (i, surql_param) in surql_params.iter().enumerate() {
					if i >= rust_params.len() {
						break;
					}
					let (ref rust_name, ref rust_type_str) = rust_params[i];
					let surql_type = &surql_param.kind;

					if !surql_type_matches_rust(surql_type, rust_type_str) {
						let hint = type_hint_for_surql(surql_type);
						let msg = format!(
							"type mismatch for {name} parameter ${}: \
							 SurrealQL type is `{surql_type}` but Rust type is `{rust_type_str}`\n\
							 \x20 expected Rust types for `{surql_type}`: {hint}",
							surql_param.name
						);
						// Point to the specific parameter in the Rust function
						let span = rust_fn
							.sig
							.inputs
							.iter()
							.filter(|arg| matches!(arg, syn::FnArg::Typed(_)))
							.nth(i)
							.map(|arg| {
								use syn::spanned::Spanned;
								arg.span()
							})
							.unwrap_or_else(|| {
								use syn::spanned::Spanned;
								rust_fn.sig.span()
							});
						return syn::Error::new(span, msg).to_compile_error().into();
					}

					// If param names differ, note it (not an error, just informational)
					let _ = rust_name;
				}
			}
			Ok(None) => {
				let msg = format!(
					"DEFINE FUNCTION not found for '{name}' in schema path '{schema_path}'"
				);
				return syn::Error::new(fn_name.span(), msg)
					.to_compile_error()
					.into();
			}
			Err(e) => {
				let msg = format!("failed to scan schema files: {e}");
				return syn::Error::new(schema_lit.span(), msg)
					.to_compile_error()
					.into();
			}
		}
	}

	// Return the function as-is with a doc attribute
	let item = proc_macro2::TokenStream::from(item);
	let doc = format!(" SurrealQL function: `{name}`");
	quote! {
		#[doc = #doc]
		#item
	}
	.into()
}

/// Count non-self, non-receiver parameters in a Rust function signature.
fn count_fn_params(func: &syn::ItemFn) -> usize {
	func.sig
		.inputs
		.iter()
		.filter(|arg| matches!(arg, syn::FnArg::Typed(_)))
		.count()
}

/// Parsed attribute for `#[surql_function("fn::name", schema = "path/")]`.
struct SurqlFunctionAttr {
	name: LitStr,
	schema: Option<LitStr>,
}

impl syn::parse::Parse for SurqlFunctionAttr {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let name: LitStr = input.parse()?;
		let mut schema = None;

		if input.peek(Token![,]) {
			let _: Token![,] = input.parse()?;
			if !input.is_empty() {
				let key: syn::Ident = input.parse()?;
				if key != "schema" {
					return Err(syn::Error::new(
						key.span(),
						format!("unexpected attribute '{key}', expected 'schema'"),
					));
				}
				let _: Token![=] = input.parse()?;
				schema = Some(input.parse()?);
			}
		}

		Ok(SurqlFunctionAttr { name, schema })
	}
}
