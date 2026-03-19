//! Error-recovering parser — parses valid statements even when some are broken.
//!
//! Splits the source into statement chunks by scanning for `;` tokens,
//! then parses each chunk independently. Returns partial results (successful
//! ASTs) alongside diagnostics for failed chunks.
//!
//! This is essential for LSP — the document is *always* invalid while typing,
//! but the LSP needs to provide completions/hover for the valid parts.
//!
//! # Example
//!
//! ```
//! let (stmts, diags) = surql_parser::parse_with_recovery(
//!     "SELECT * FROM user; SELEC broken; DEFINE TABLE post SCHEMAFULL"
//! );
//! assert_eq!(stmts.len(), 2); // first and third succeeded
//! assert_eq!(diags.len(), 1); // second failed
//! ```

use crate::ParseDiagnostic;
use crate::upstream::sql::ast::TopLevelExpr;
use crate::upstream::syn::lexer::Lexer;
use crate::upstream::syn::token::{Delim, TokenKind};

/// Parse a SurrealQL document with error recovery.
///
/// Returns a tuple of:
/// - Successfully parsed statements
/// - Diagnostics for statements that failed to parse
pub fn parse_with_recovery(source: &str) -> (Vec<TopLevelExpr>, Vec<ParseDiagnostic>) {
	if source.trim().is_empty() {
		return (Vec::new(), Vec::new());
	}

	let chunks = split_into_chunks(source);
	let mut all_stmts = Vec::new();
	let mut all_diags = Vec::new();

	for chunk in &chunks {
		let chunk_source = &source[chunk.start..chunk.end];
		if chunk_source.trim().is_empty() {
			continue;
		}

		match crate::parse(chunk_source) {
			Ok(ast) => {
				all_stmts.extend(ast.expressions);
			}
			Err(_) => {
				// Try parse_for_diagnostics for precise error info
				if let Err(diags) = crate::parse_for_diagnostics(chunk_source) {
					for mut d in diags {
						// Adjust line/column offsets for the chunk position in the original source
						let (base_line, base_col) = byte_offset_to_line_col(source, chunk.start);
						if d.line == 1 {
							d.column += base_col;
							d.end_column += base_col;
						}
						d.line += base_line;
						d.end_line += base_line;
						all_diags.push(d);
					}
				} else {
					// parse failed but parse_for_diagnostics succeeded? Shouldn't happen.
					// Add a generic diagnostic.
					let (line, col) = byte_offset_to_line_col(source, chunk.start);
					all_diags.push(ParseDiagnostic {
						message: "Parse error".into(),
						line: line + 1,
						column: col + 1,
						end_line: line + 1,
						end_column: col + 1,
					});
				}
			}
		}
	}

	(all_stmts, all_diags)
}

/// A byte range representing a statement chunk in the source.
struct Chunk {
	start: usize,
	end: usize,
}

/// Split source into statement chunks by tokenizing and finding `;` boundaries.
/// Respects brace depth — semicolons inside `{ }` blocks are not split points.
fn split_into_chunks(source: &str) -> Vec<Chunk> {
	let bytes = source.as_bytes();
	if bytes.len() > u32::MAX as usize {
		return vec![Chunk {
			start: 0,
			end: source.len(),
		}];
	}

	let lexer = Lexer::new(bytes);
	let mut chunks = Vec::new();
	let mut chunk_start = 0;
	let mut brace_depth: u32 = 0;

	for token in lexer {
		match token.kind {
			TokenKind::OpenDelim(Delim::Brace) => brace_depth += 1,
			TokenKind::CloseDelim(Delim::Brace) => brace_depth = brace_depth.saturating_sub(1),
			TokenKind::SemiColon if brace_depth == 0 => {
				let semi_end = token.span.offset as usize + token.span.len as usize;
				if chunk_start < token.span.offset as usize {
					chunks.push(Chunk {
						start: chunk_start,
						end: token.span.offset as usize,
					});
				}
				chunk_start = semi_end;
			}
			_ => {}
		}
	}

	// Last chunk (after final semicolon or if no semicolons)
	if chunk_start < source.len() {
		let remaining = source[chunk_start..].trim();
		if !remaining.is_empty() {
			chunks.push(Chunk {
				start: chunk_start,
				end: source.len(),
			});
		}
	}

	// If no semicolons found, treat entire source as one chunk
	if chunks.is_empty() && !source.trim().is_empty() {
		chunks.push(Chunk {
			start: 0,
			end: source.len(),
		});
	}

	chunks
}

/// Convert a byte offset to (0-indexed line, 0-indexed column).
fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
	let offset = offset.min(source.len());
	let before = &source[..offset];
	let line = before.matches('\n').count();
	let col = before.rfind('\n').map(|i| offset - i - 1).unwrap_or(offset);
	(line, col)
}
