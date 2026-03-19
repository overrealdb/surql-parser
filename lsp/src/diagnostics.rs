//! Convert surql-parser diagnostics to LSP diagnostics.

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

/// Parse a SurrealQL document and return LSP diagnostics.
pub fn compute(source: &str) -> Vec<Diagnostic> {
	match surql_parser::parse_for_diagnostics(source) {
		Ok(_) => vec![],
		Err(diags) => diags
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
			.collect(),
	}
}
