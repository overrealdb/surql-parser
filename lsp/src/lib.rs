//! SurrealQL Language Server — library interface for testing.

pub mod completion;
pub mod context;
pub mod diagnostics;
pub mod document;
pub mod embedded;
pub mod embedded_db;
pub mod formatting;
pub mod keywords;
pub mod server;
pub mod signature;

#[cfg(test)]
mod tests;
