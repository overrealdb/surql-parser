//! SurrealQL Language Server — library interface for testing.

pub mod completion;
pub mod context;
pub mod diagnostics;
pub mod document;
mod dotenv;
pub mod embedded;
pub mod embedded_db;
pub mod formatting;
mod hover;
pub mod keywords;
mod manifest;
pub mod server;
pub mod signature;

#[cfg(test)]
mod tests;
