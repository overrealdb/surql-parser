//! Convert surql-parser diagnostics to LSP diagnostics.
//!
//! Uses error-recovering parser so diagnostics are reported for ALL broken
//! statements, not just the first one.

use surql_parser::upstream::sql::ast::TopLevelExpr;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

/// Result of parsing a document: partial AST + diagnostics.
pub struct DocumentParseResult {
	pub statements: Vec<TopLevelExpr>,
	pub diagnostics: Vec<Diagnostic>,
}

/// Parse a SurrealQL document with error recovery.
///
/// Returns both the successfully parsed statements (for schema/completions)
/// and diagnostics for broken statements.
pub fn compute_with_recovery(source: &str) -> DocumentParseResult {
	let (stmts, diags) = surql_parser::parse_with_recovery(source);
	let diagnostics = diags
		.into_iter()
		.map(|d| Diagnostic {
			range: Range {
				start: Position {
					line: (d.line.saturating_sub(1)) as u32,
					character: (d.column.saturating_sub(1)) as u32,
				},
				end: Position {
					line: (d.end_line.saturating_sub(1)) as u32,
					character: (d.end_column.saturating_sub(1)) as u32,
				},
			},
			severity: Some(DiagnosticSeverity::ERROR),
			source: Some("surql".into()),
			message: d.message,
			..Default::default()
		})
		.collect();
	DocumentParseResult {
		statements: stmts,
		diagnostics,
	}
}

/// Simple diagnostic computation (backwards compat for tests).
pub fn compute(source: &str) -> Vec<Diagnostic> {
	compute_with_recovery(source).diagnostics
}
