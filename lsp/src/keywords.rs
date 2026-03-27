//! SurrealQL keyword list for completions.
//!
//! Uses surql_parser's keyword list to stay in sync with the parser.

pub fn all_keywords() -> &'static [&'static str] {
	// This list is verified against the parser's Keyword enum by
	// tests/keywords_sync.rs in the surql-parser crate.
	surql_parser::all_keywords()
}
