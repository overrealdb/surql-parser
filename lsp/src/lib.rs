//! SurrealQL Language Server — library interface for testing.

pub mod completion;
pub mod diagnostics;
pub mod document;
pub mod formatting;
pub mod keywords;
pub mod server;
pub mod signature;

#[cfg(test)]
mod tests;
