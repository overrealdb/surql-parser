//! Compile-time SurrealQL validation macros.
//!
//! Provides proc macros that validate SurrealQL at compile time using `surql-parser`.
//!
//! # Macros
//!
//! - [`surql_check!`] — validate a SurrealQL string at compile time
//! - [`#[surql_function]`](macro@surql_function) — validate a SurrealQL function name at compile time

use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr, parse_macro_input};

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

/// Validates a SurrealQL function name at compile time.
///
/// Place this attribute on a Rust function with a string literal argument
/// like `"fn::get_entity"`. The macro validates at compile time that:
///
/// 1. The name starts with `fn::`
/// 2. The name is syntactically valid as a SurrealQL function call
///
/// The annotated function is preserved as-is (the macro only adds a doc comment).
///
/// # Example
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
	let fn_name = parse_macro_input!(attr as LitStr);
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

	// Return the function as-is with a doc attribute
	let item = proc_macro2::TokenStream::from(item);
	let doc = format!(" SurrealQL function: `{name}`");
	quote! {
		#[doc = #doc]
		#item
	}
	.into()
}
