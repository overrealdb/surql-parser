//! Configuration constants for the parser.
//!
//! These replace `crate::cnf::*` from the SurrealDB engine.

/// Maximum depth for parsing nested queries (subqueries).
pub const MAX_QUERY_PARSING_DEPTH: u32 = 20;

/// Maximum depth for parsing nested objects/arrays.
pub const MAX_OBJECT_PARSING_DEPTH: u32 = 100;
