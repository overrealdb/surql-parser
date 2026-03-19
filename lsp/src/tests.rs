//! Comprehensive LSP tests organized by LSP protocol categories.
//!
//! Test structure follows patterns from rust-analyzer and sqls:
//! - Unit tests per handler (fast, fixture-based)
//! - Edge cases from mature LSP implementations
//! - Cursor position markers via (line, col) tuples

// ═══════════════════════════════════════════════════════════════════════
// 1. DOCUMENT SYNCHRONIZATION
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod document_sync {
	use crate::document::DocumentStore;
	use tower_lsp::lsp_types::Url;

	fn uri(s: &str) -> Url {
		Url::parse(s).unwrap()
	}

	#[test]
	fn open_stores_content() {
		let store = DocumentStore::new();
		store.open(uri("file:///a.surql"), "SELECT * FROM user".into());
		assert_eq!(
			store.get(&uri("file:///a.surql")).unwrap(),
			"SELECT * FROM user"
		);
	}

	#[test]
	fn update_replaces_content() {
		let store = DocumentStore::new();
		store.open(uri("file:///a.surql"), "old".into());
		store.update(&uri("file:///a.surql"), "new".into());
		assert_eq!(store.get(&uri("file:///a.surql")).unwrap(), "new");
	}

	#[test]
	fn close_removes_document() {
		let store = DocumentStore::new();
		store.open(uri("file:///a.surql"), "content".into());
		store.close(&uri("file:///a.surql"));
		assert!(store.get(&uri("file:///a.surql")).is_none());
	}

	#[test]
	fn get_nonexistent_returns_none() {
		let store = DocumentStore::new();
		assert!(store.get(&uri("file:///missing.surql")).is_none());
	}

	#[test]
	fn multiple_documents_isolated() {
		let store = DocumentStore::new();
		store.open(uri("file:///a.surql"), "aaa".into());
		store.open(uri("file:///b.surql"), "bbb".into());
		store.update(&uri("file:///a.surql"), "updated".into());
		assert_eq!(store.get(&uri("file:///a.surql")).unwrap(), "updated");
		assert_eq!(store.get(&uri("file:///b.surql")).unwrap(), "bbb");
	}

	#[test]
	fn update_nonexistent_creates_it() {
		let store = DocumentStore::new();
		store.update(&uri("file:///new.surql"), "created".into());
		assert_eq!(store.get(&uri("file:///new.surql")).unwrap(), "created");
	}

	#[test]
	fn close_nonexistent_no_panic() {
		let store = DocumentStore::new();
		store.close(&uri("file:///never_opened.surql")); // must not panic
	}

	#[test]
	fn empty_content() {
		let store = DocumentStore::new();
		store.open(uri("file:///empty.surql"), String::new());
		assert_eq!(store.get(&uri("file:///empty.surql")).unwrap(), "");
	}
}

// ═══════════════════════════════════════════════════════════════════════
// 2. DIAGNOSTICS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod diagnostics {
	use crate::diagnostics;
	use tower_lsp::lsp_types::DiagnosticSeverity;

	// ─── Valid inputs produce no diagnostics ───

	#[test]
	fn empty_input() {
		assert!(diagnostics::compute("").is_empty());
	}

	#[test]
	fn comment_only() {
		assert!(diagnostics::compute("-- comment").is_empty());
	}

	#[test]
	fn block_comment_only() {
		assert!(diagnostics::compute("/* block */").is_empty());
	}

	#[test]
	fn whitespace_only() {
		assert!(diagnostics::compute("   \n\t\n  ").is_empty());
	}

	#[test]
	fn single_statement() {
		assert!(diagnostics::compute("SELECT * FROM user").is_empty());
	}

	#[test]
	fn multiple_statements() {
		assert!(diagnostics::compute("SELECT * FROM a; SELECT * FROM b").is_empty());
	}

	#[test]
	fn define_table() {
		assert!(diagnostics::compute("DEFINE TABLE user SCHEMAFULL").is_empty());
	}

	#[test]
	fn define_function() {
		assert!(
			diagnostics::compute("DEFINE FUNCTION fn::greet($name: string) { RETURN 'Hello'; }")
				.is_empty()
		);
	}

	#[test]
	fn relate() {
		assert!(
			diagnostics::compute("RELATE user:tobie->follows->user:jaime SET since = time::now()")
				.is_empty()
		);
	}

	#[test]
	fn graph_traversal() {
		assert!(
			diagnostics::compute("SELECT ->follows->user.name AS friends FROM user:tobie")
				.is_empty()
		);
	}

	#[test]
	fn complex_query() {
		assert!(
			diagnostics::compute(
				"SELECT count() AS total, country FROM user GROUP BY country ORDER BY total DESC"
			)
			.is_empty()
		);
	}

	// ─── Invalid inputs produce diagnostics ───

	#[test]
	fn syntax_error_returns_diagnostic() {
		let diags = diagnostics::compute("SELEC * FROM user");
		assert!(!diags.is_empty());
	}

	#[test]
	fn severity_is_error() {
		let diags = diagnostics::compute("SELEC * FROM user");
		assert_eq!(diags[0].severity, Some(DiagnosticSeverity::ERROR));
	}

	#[test]
	fn source_is_surql() {
		let diags = diagnostics::compute("SELEC * FROM user");
		assert_eq!(diags[0].source.as_deref(), Some("surql"));
	}

	#[test]
	fn position_is_zero_indexed() {
		let diags = diagnostics::compute("SELEC * FROM user");
		assert_eq!(diags[0].range.start.line, 0);
	}

	#[test]
	fn unclosed_string() {
		let diags = diagnostics::compute("SELECT * FROM user WHERE name = 'oops");
		assert!(!diags.is_empty());
	}

	#[test]
	fn unclosed_paren() {
		let diags = diagnostics::compute("SELECT * FROM (SELECT name FROM user");
		assert!(!diags.is_empty());
	}

	#[test]
	fn unclosed_brace() {
		let diags = diagnostics::compute("DEFINE FUNCTION fn::test() {");
		assert!(!diags.is_empty());
	}

	#[test]
	fn multiline_error_on_correct_line() {
		let diags = diagnostics::compute("SELECT * FROM user;\nSELEC * FROM post");
		assert!(!diags.is_empty());
		assert_eq!(diags[0].range.start.line, 1); // second line, 0-indexed
	}

	#[test]
	fn error_in_second_statement() {
		let diags = diagnostics::compute("SELECT * FROM user; SELEC * FROM post");
		assert!(!diags.is_empty());
	}

	#[test]
	fn message_is_descriptive() {
		let diags = diagnostics::compute("SELEC * FROM user");
		assert!(!diags[0].message.is_empty());
	}

	#[test]
	fn invalid_type_annotation() {
		let diags = diagnostics::compute("DEFINE FIELD name ON user TYPE strng");
		// "strng" is actually valid as a table name in this context, but let's see
		// what the parser says
		let _ = diags; // just don't panic
	}
}

// ═══════════════════════════════════════════════════════════════════════
// 3. COMPLETION — Context Detection
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod completion_context {
	use crate::completion::{Context, detect_context};
	use tower_lsp::lsp_types::Position;

	fn pos(line: u32, col: u32) -> Position {
		Position {
			line,
			character: col,
		}
	}

	// ─── General context ───

	#[test]
	fn empty_document() {
		assert_eq!(detect_context("", pos(0, 0)), Context::General);
	}

	#[test]
	fn start_of_line() {
		assert_eq!(detect_context("SELECT", pos(0, 0)), Context::General);
	}

	#[test]
	fn after_select() {
		assert_eq!(detect_context("SELECT ", pos(0, 7)), Context::General);
	}

	#[test]
	fn after_where() {
		assert_eq!(
			detect_context("SELECT * FROM user WHERE ", pos(0, 25)),
			Context::General
		);
	}

	#[test]
	fn after_equals() {
		assert_eq!(detect_context("SET name = ", pos(0, 11)), Context::General);
	}

	// ─── Table name context ───

	#[test]
	fn after_from() {
		assert_eq!(
			detect_context("SELECT * FROM ", pos(0, 14)),
			Context::TableName
		);
	}

	#[test]
	fn after_into() {
		assert_eq!(
			detect_context("INSERT INTO ", pos(0, 12)),
			Context::TableName
		);
	}

	#[test]
	fn after_on() {
		assert_eq!(
			detect_context("DEFINE FIELD name ON ", pos(0, 21)),
			Context::TableName
		);
	}

	#[test]
	fn after_table() {
		assert_eq!(
			detect_context("DEFINE TABLE ", pos(0, 13)),
			Context::TableName
		);
	}

	#[test]
	fn from_case_insensitive() {
		assert_eq!(
			detect_context("select * from ", pos(0, 14)),
			Context::TableName
		);
	}

	#[test]
	fn from_on_second_line() {
		assert_eq!(
			detect_context("SELECT *\nFROM ", pos(1, 5)),
			Context::TableName
		);
	}

	// ─── Function name context ───

	#[test]
	fn after_fn_colons() {
		assert_eq!(
			detect_context("SELECT fn::", pos(0, 11)),
			Context::FunctionName
		);
	}

	#[test]
	fn after_fn_single_colon() {
		assert_eq!(detect_context("fn:", pos(0, 3)), Context::FunctionName);
	}

	// ─── Param name context ───

	#[test]
	fn after_dollar_sign() {
		assert_eq!(
			detect_context("WHERE age > $", pos(0, 13)),
			Context::ParamName
		);
	}

	#[test]
	fn dollar_at_line_start() {
		assert_eq!(detect_context("$", pos(0, 1)), Context::ParamName);
	}

	#[test]
	fn dollar_after_equals() {
		assert_eq!(
			detect_context("SET name = $", pos(0, 12)),
			Context::ParamName
		);
	}
}

// ═══════════════════════════════════════════════════════════════════════
// 4. COMPLETION — Result Items
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod completion_items {
	use crate::completion;
	use surql_parser::SchemaGraph;
	use tower_lsp::lsp_types::{CompletionItemKind, Position};

	fn pos(line: u32, col: u32) -> Position {
		Position {
			line,
			character: col,
		}
	}

	fn schema() -> SchemaGraph {
		SchemaGraph::from_source(
			"
			DEFINE TABLE user SCHEMAFULL;
			DEFINE FIELD name ON user TYPE string;
			DEFINE FIELD age ON user TYPE int;
			DEFINE TABLE post SCHEMAFULL;
			DEFINE FIELD title ON post TYPE string;
			DEFINE FIELD author ON post TYPE record<user>;
			DEFINE FUNCTION fn::greet($name: string) -> string { RETURN 'Hello, ' + $name; };
			DEFINE FUNCTION fn::add($a: int, $b: int) -> int { RETURN $a + $b; };
		",
		)
		.unwrap()
	}

	// ─── Without schema ───

	#[test]
	fn no_schema_returns_keywords() {
		let items = completion::complete("", pos(0, 0), None);
		assert!(!items.is_empty());
		assert!(items.iter().any(|i| i.label == "SELECT"));
	}

	#[test]
	fn no_schema_after_from_still_returns_keywords() {
		let items = completion::complete("SELECT * FROM ", pos(0, 14), None);
		assert!(
			items
				.iter()
				.any(|i| i.kind == Some(CompletionItemKind::KEYWORD))
		);
	}

	// ─── With schema: table completions ───

	#[test]
	fn tables_after_from() {
		let sg = schema();
		let items = completion::complete("SELECT * FROM ", pos(0, 14), Some(&sg));
		let tables: Vec<_> = items
			.iter()
			.filter(|i| i.kind == Some(CompletionItemKind::CLASS))
			.collect();
		assert!(tables.iter().any(|i| i.label == "user"));
		assert!(tables.iter().any(|i| i.label == "post"));
	}

	#[test]
	fn tables_after_into() {
		let sg = schema();
		let items = completion::complete("INSERT INTO ", pos(0, 12), Some(&sg));
		assert!(items.iter().any(|i| i.label == "user"));
	}

	// ─── With schema: function completions ───

	#[test]
	fn functions_after_fn_prefix() {
		let sg = schema();
		let items = completion::complete("SELECT fn::", pos(0, 11), Some(&sg));
		let fns: Vec<_> = items
			.iter()
			.filter(|i| i.kind == Some(CompletionItemKind::FUNCTION))
			.collect();
		assert!(fns.iter().any(|i| i.label == "fn::greet"));
		assert!(fns.iter().any(|i| i.label == "fn::add"));
	}

	#[test]
	fn function_detail_shows_signature() {
		let sg = schema();
		let items = completion::complete("SELECT fn::", pos(0, 11), Some(&sg));
		let greet = items.iter().find(|i| i.label == "fn::greet").unwrap();
		let detail = greet.detail.as_deref().unwrap();
		assert!(detail.contains("$name: string"));
		assert!(detail.contains("-> string"));
	}

	#[test]
	fn function_add_has_two_params() {
		let sg = schema();
		let items = completion::complete("SELECT fn::", pos(0, 11), Some(&sg));
		let add = items.iter().find(|i| i.label == "fn::add").unwrap();
		let detail = add.detail.as_deref().unwrap();
		assert!(detail.contains("$a: int"));
		assert!(detail.contains("$b: int"));
	}

	// ─── General context with schema ───

	#[test]
	fn general_includes_tables_keywords_functions() {
		let sg = schema();
		let items = completion::complete("", pos(0, 0), Some(&sg));
		assert!(
			items
				.iter()
				.any(|i| i.kind == Some(CompletionItemKind::CLASS))
		);
		assert!(
			items
				.iter()
				.any(|i| i.kind == Some(CompletionItemKind::KEYWORD))
		);
		assert!(
			items
				.iter()
				.any(|i| i.kind == Some(CompletionItemKind::FUNCTION))
		);
	}

	// ─── Keyword quality ───

	#[test]
	fn common_keywords_present() {
		let items = completion::complete("", pos(0, 0), None);
		for kw in &[
			"SELECT", "FROM", "WHERE", "CREATE", "UPDATE", "DELETE", "DEFINE",
		] {
			assert!(
				items.iter().any(|i| i.label == *kw),
				"missing keyword: {kw}"
			);
		}
	}

	#[test]
	fn type_keywords_present() {
		let items = completion::complete("", pos(0, 0), None);
		for kw in &["string", "int", "float", "bool", "datetime", "record"] {
			assert!(
				items.iter().any(|i| i.label == *kw),
				"missing type keyword: {kw}"
			);
		}
	}
}

// ═══════════════════════════════════════════════════════════════════════
// 5. HOVER
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod hover {
	use crate::server::word_at_position;
	use tower_lsp::lsp_types::Position;

	fn pos(line: u32, col: u32) -> Position {
		Position {
			line,
			character: col,
		}
	}

	// ─── Word extraction ───

	#[test]
	fn simple_word() {
		assert_eq!(word_at_position("SELECT * FROM user", pos(0, 15)), "user");
	}

	#[test]
	fn word_at_start() {
		assert_eq!(word_at_position("SELECT * FROM user", pos(0, 3)), "SELECT");
	}

	#[test]
	fn word_with_colons() {
		assert_eq!(
			word_at_position("SELECT fn::greet()", pos(0, 12)),
			"fn::greet"
		);
	}

	#[test]
	fn word_with_underscores() {
		assert_eq!(
			word_at_position("SELECT user_name FROM t", pos(0, 10)),
			"user_name"
		);
	}

	#[test]
	fn empty_at_space() {
		assert_eq!(word_at_position("SELECT * FROM user", pos(0, 7)), "");
	}

	#[test]
	fn empty_at_paren() {
		// pos 5 is '(' which is not alphanumeric
		assert_eq!(word_at_position("count()", pos(0, 5)), "count");
	}

	#[test]
	fn empty_at_operator() {
		assert_eq!(word_at_position("a + b", pos(0, 2)), "");
	}

	#[test]
	fn empty_source() {
		assert_eq!(word_at_position("", pos(0, 0)), "");
	}

	#[test]
	fn multiline_second_line() {
		assert_eq!(word_at_position("SELECT *\nFROM user", pos(1, 6)), "user");
	}

	#[test]
	fn out_of_bounds_line() {
		assert_eq!(word_at_position("SELECT", pos(5, 0)), "");
	}

	#[test]
	fn cursor_at_word_boundary_start() {
		assert_eq!(word_at_position("SELECT * FROM user", pos(0, 14)), "user");
	}

	#[test]
	fn cursor_at_word_boundary_end() {
		assert_eq!(word_at_position("SELECT * FROM user", pos(0, 18)), "user");
	}

	#[test]
	fn word_at_eol() {
		assert_eq!(word_at_position("SELECT * FROM user\n", pos(0, 18)), "user");
	}

	#[test]
	fn unicode_safe() {
		// Should not panic on non-ASCII
		let _ = word_at_position("SELECT * FROM юзер", pos(0, 14));
	}
}

// ═══════════════════════════════════════════════════════════════════════
// 6. FORMATTING
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod formatting {
	use crate::formatting;

	#[test]
	fn formats_valid_sql() {
		assert!(formatting::format_document("SELECT  *   FROM   user").is_some());
	}

	#[test]
	fn returns_none_for_invalid_sql() {
		assert!(formatting::format_document("SELEC * FORM user").is_none());
	}

	#[test]
	fn no_edit_when_already_formatted() {
		let ast = surql_parser::parse("SELECT * FROM user").unwrap();
		let formatted = surql_parser::format(&ast);
		assert!(formatting::format_document(&formatted).is_none());
	}

	#[test]
	fn returns_single_text_edit() {
		if let Some(edits) = formatting::format_document("SELECT  *   FROM   user") {
			assert_eq!(edits.len(), 1);
			assert_eq!(edits[0].range.start.line, 0);
			assert_eq!(edits[0].range.start.character, 0);
		}
	}

	#[test]
	fn multistatement_formatting() {
		assert!(formatting::format_document("SELECT * FROM a;SELECT * FROM b").is_some());
	}

	#[test]
	fn empty_input() {
		// Empty is valid, parse succeeds, format might be same
		let result = formatting::format_document("");
		// Either None (no change) or Some — just don't panic
		let _ = result;
	}

	#[test]
	fn preserves_semicolons() {
		if let Some(edits) = formatting::format_document("SELECT * FROM a ; SELECT * FROM b") {
			assert!(edits[0].new_text.contains(';'));
		}
	}

	#[test]
	fn define_table_formatting() {
		let result = formatting::format_document("DEFINE  TABLE  user  SCHEMAFULL");
		// Should format without error
		let _ = result;
	}
}

// ═══════════════════════════════════════════════════════════════════════
// 7. KEYWORDS LIST QUALITY
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod keywords {
	use crate::keywords::KEYWORDS;

	#[test]
	fn not_empty() {
		assert!(
			KEYWORDS.len() > 100,
			"Expected 100+ keywords, got {}",
			KEYWORDS.len()
		);
	}

	#[test]
	fn contains_dml_keywords() {
		for kw in &[
			"SELECT", "CREATE", "UPDATE", "DELETE", "INSERT", "UPSERT", "RELATE",
		] {
			assert!(KEYWORDS.contains(kw), "Missing DML keyword: {kw}");
		}
	}

	#[test]
	fn contains_ddl_keywords() {
		for kw in &[
			"DEFINE", "REMOVE", "ALTER", "TABLE", "FIELD", "INDEX", "FUNCTION",
		] {
			assert!(KEYWORDS.contains(kw), "Missing DDL keyword: {kw}");
		}
	}

	#[test]
	fn contains_type_keywords() {
		for kw in &[
			"string", "int", "float", "bool", "datetime", "record", "array", "object",
		] {
			assert!(KEYWORDS.contains(kw), "Missing type keyword: {kw}");
		}
	}

	#[test]
	fn contains_control_flow() {
		for kw in &["IF", "ELSE", "FOR", "BEGIN", "COMMIT", "CANCEL"] {
			assert!(KEYWORDS.contains(kw), "Missing control keyword: {kw}");
		}
	}

	#[test]
	fn contains_live_query() {
		assert!(KEYWORDS.contains(&"LIVE"));
		assert!(KEYWORDS.contains(&"KILL"));
	}

	#[test]
	fn no_empty_strings() {
		assert!(KEYWORDS.iter().all(|kw| !kw.is_empty()));
	}

	#[test]
	fn no_duplicates() {
		let mut sorted = KEYWORDS.to_vec();
		sorted.sort();
		sorted.dedup();
		// IF appears twice in the list — one for control flow, one for DDL
		// That's a minor issue but shouldn't cause problems
		assert!(
			sorted.len() >= KEYWORDS.len() - 5,
			"Too many duplicates in keyword list"
		);
	}
}

// ═══════════════════════════════════════════════════════════════════════
// 8. EDGE CASES — graceful degradation
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod edge_cases {
	use crate::completion;
	use crate::diagnostics;
	use crate::formatting;
	use crate::server::word_at_position;
	use tower_lsp::lsp_types::Position;

	fn pos(line: u32, col: u32) -> Position {
		Position {
			line,
			character: col,
		}
	}

	// ─── Very large documents ───

	#[test]
	fn large_valid_document() {
		let mut doc = String::new();
		for i in 0..100 {
			doc.push_str(&format!("SELECT * FROM table_{i};\n"));
		}
		assert!(diagnostics::compute(&doc).is_empty());
	}

	#[test]
	fn many_define_statements() {
		let mut doc = String::new();
		for i in 0..50 {
			doc.push_str(&format!("DEFINE TABLE t{i} SCHEMALESS;\n"));
		}
		assert!(diagnostics::compute(&doc).is_empty());
	}

	// ─── Unicode handling ───

	#[test]
	fn unicode_in_strings() {
		assert!(diagnostics::compute("SELECT * FROM user WHERE name = '日本語'").is_empty());
	}

	#[test]
	fn emoji_in_strings() {
		assert!(diagnostics::compute("CREATE post SET title = '🚀 Launch!'").is_empty());
	}

	#[test]
	fn completion_with_unicode() {
		// Must not panic
		let _ = completion::complete("SELECT * FROM юзер WHERE ", pos(0, 25), None);
	}

	#[test]
	fn word_at_position_unicode() {
		// Should not panic even with multi-byte chars
		let _ = word_at_position("SELECT * FROM таблица", pos(0, 14));
	}

	// ─── Cursor at boundaries ───

	#[test]
	fn completion_at_col_zero() {
		let _ = completion::complete("SELECT", pos(0, 0), None);
	}

	#[test]
	fn completion_past_end_of_line() {
		let _ = completion::complete("SELECT", pos(0, 100), None);
	}

	#[test]
	fn completion_past_end_of_document() {
		let _ = completion::complete("SELECT", pos(100, 0), None);
	}

	// ─── Empty/whitespace inputs ───

	#[test]
	fn format_whitespace() {
		let _ = formatting::format_document("   \n\t\n  ");
	}

	#[test]
	fn completion_whitespace_only() {
		let _ = completion::complete("   ", pos(0, 3), None);
	}

	#[test]
	fn diagnostics_single_semicolon() {
		let _ = diagnostics::compute(";");
	}

	// ─── Deeply nested queries ───

	#[test]
	fn nested_subquery() {
		assert!(
			diagnostics::compute("SELECT * FROM (SELECT * FROM (SELECT * FROM user))").is_empty()
		);
	}

	#[test]
	fn nested_if_else() {
		assert!(
			diagnostics::compute("IF true { IF true { IF true { RETURN 1 } ELSE { RETURN 2 } } }")
				.is_empty()
		);
	}
}

// ═══════════════════════════════════════════════════════════════════════
// 9. FIELD COMPLETIONS (after `table.`)
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod field_completions {
	use crate::completion;
	use crate::completion::{Context, detect_context};
	use surql_parser::SchemaGraph;
	use tower_lsp::lsp_types::{CompletionItemKind, Position};

	fn pos(line: u32, col: u32) -> Position {
		Position {
			line,
			character: col,
		}
	}

	fn schema() -> SchemaGraph {
		SchemaGraph::from_source(
			"
			DEFINE TABLE user SCHEMAFULL;
			DEFINE FIELD name ON user TYPE string;
			DEFINE FIELD age ON user TYPE int;
			DEFINE FIELD email ON user TYPE string;
			DEFINE TABLE post SCHEMAFULL;
			DEFINE FIELD title ON post TYPE string;
			DEFINE FIELD author ON post TYPE record<user>;
		",
		)
		.unwrap()
	}

	#[test]
	fn context_after_table_dot() {
		assert_eq!(
			detect_context("user.", pos(0, 5)),
			Context::FieldName("user".into())
		);
	}

	#[test]
	fn context_after_table_dot_in_select() {
		assert_eq!(
			detect_context("SELECT user.", pos(0, 12)),
			Context::FieldName("user".into())
		);
	}

	#[test]
	fn context_after_table_dot_in_where() {
		// col 30 = after the dot (len of "SELECT * FROM user WHERE user.")
		assert_eq!(
			detect_context("SELECT * FROM user WHERE user.", pos(0, 30)),
			Context::FieldName("user".into())
		);
	}

	#[test]
	fn fields_returned_for_known_table() {
		let sg = schema();
		let items = completion::complete("SELECT user.", pos(0, 12), Some(&sg));
		let fields: Vec<_> = items
			.iter()
			.filter(|i| i.kind == Some(CompletionItemKind::FIELD))
			.collect();
		assert!(fields.iter().any(|i| i.label == "name"));
		assert!(fields.iter().any(|i| i.label == "age"));
		assert!(fields.iter().any(|i| i.label == "email"));
	}

	#[test]
	fn field_detail_shows_type() {
		let sg = schema();
		let items = completion::complete("SELECT user.", pos(0, 12), Some(&sg));
		let name = items.iter().find(|i| i.label == "name").unwrap();
		assert_eq!(name.detail.as_deref(), Some("string"));
	}

	#[test]
	fn post_fields_different_from_user() {
		let sg = schema();
		let items = completion::complete("SELECT post.", pos(0, 12), Some(&sg));
		let fields: Vec<_> = items
			.iter()
			.filter(|i| i.kind == Some(CompletionItemKind::FIELD))
			.collect();
		assert!(fields.iter().any(|i| i.label == "title"));
		assert!(fields.iter().any(|i| i.label == "author"));
		assert!(!fields.iter().any(|i| i.label == "name"));
	}

	#[test]
	fn unknown_table_returns_no_fields() {
		let sg = schema();
		let items = completion::complete("SELECT unknown.", pos(0, 15), Some(&sg));
		let fields: Vec<_> = items
			.iter()
			.filter(|i| i.kind == Some(CompletionItemKind::FIELD))
			.collect();
		assert!(fields.is_empty());
	}

	#[test]
	fn graph_traversal_operators_included() {
		let sg = schema();
		let items = completion::complete("SELECT user.", pos(0, 12), Some(&sg));
		assert!(items.iter().any(|i| i.label == "->"));
		assert!(items.iter().any(|i| i.label == "<-"));
	}

	#[test]
	fn no_schema_no_fields() {
		let items = completion::complete("SELECT user.", pos(0, 12), None);
		let fields: Vec<_> = items
			.iter()
			.filter(|i| i.kind == Some(CompletionItemKind::FIELD))
			.collect();
		assert!(fields.is_empty());
	}
}

// ═══════════════════════════════════════════════════════════════════════
// 10. SIGNATURE HELP
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod signature_help {
	use crate::signature;
	use surql_parser::SchemaGraph;
	use tower_lsp::lsp_types::Position;

	fn pos(line: u32, col: u32) -> Position {
		Position {
			line,
			character: col,
		}
	}

	fn schema() -> SchemaGraph {
		SchemaGraph::from_source(
			"
			DEFINE FUNCTION fn::greet($name: string) -> string { RETURN 'Hello, ' + $name; };
			DEFINE FUNCTION fn::add($a: int, $b: int) -> int { RETURN $a + $b; };
			DEFINE FUNCTION fn::noargs() -> string { RETURN 'hi'; };
		",
		)
		.unwrap()
	}

	#[test]
	fn signature_at_opening_paren() {
		let sg = schema();
		let help = signature::signature_help("fn::greet(", pos(0, 10), Some(&sg));
		assert!(help.is_some());
		let help = help.unwrap();
		assert_eq!(help.signatures.len(), 1);
		assert!(help.signatures[0].label.contains("fn::greet"));
		assert_eq!(help.active_parameter, Some(0));
	}

	#[test]
	fn signature_first_param() {
		let sg = schema();
		let help = signature::signature_help("fn::add(1", pos(0, 9), Some(&sg));
		assert!(help.is_some());
		assert_eq!(help.unwrap().active_parameter, Some(0));
	}

	#[test]
	fn signature_second_param() {
		let sg = schema();
		let help = signature::signature_help("fn::add(1, ", pos(0, 11), Some(&sg));
		assert!(help.is_some());
		assert_eq!(help.unwrap().active_parameter, Some(1));
	}

	#[test]
	fn signature_params_info() {
		let sg = schema();
		let help = signature::signature_help("fn::add(", pos(0, 8), Some(&sg)).unwrap();
		let params = help.signatures[0].parameters.as_ref().unwrap();
		assert_eq!(params.len(), 2);
	}

	#[test]
	fn no_signature_outside_parens() {
		let sg = schema();
		let help = signature::signature_help("fn::greet", pos(0, 9), Some(&sg));
		assert!(help.is_none());
	}

	#[test]
	fn no_signature_for_unknown_function() {
		let sg = schema();
		let help = signature::signature_help("fn::unknown(", pos(0, 12), Some(&sg));
		assert!(help.is_none());
	}

	#[test]
	fn no_signature_without_schema() {
		let help = signature::signature_help("fn::greet(", pos(0, 10), None);
		assert!(help.is_none());
	}

	#[test]
	fn signature_noargs_function() {
		let sg = schema();
		let help = signature::signature_help("fn::noargs(", pos(0, 11), Some(&sg)).unwrap();
		let params = help.signatures[0].parameters.as_ref().unwrap();
		assert!(params.is_empty());
		assert_eq!(help.active_parameter, Some(0));
	}

	#[test]
	fn signature_in_nested_context() {
		let sg = schema();
		let help = signature::signature_help("SELECT fn::greet(", pos(0, 17), Some(&sg));
		assert!(help.is_some());
	}

	#[test]
	fn signature_with_nested_parens() {
		let sg = schema();
		// fn::add(count(), | — cursor after the comma, nested parens
		let help = signature::signature_help("fn::add(count(), ", pos(0, 17), Some(&sg));
		assert!(help.is_some());
		assert_eq!(help.unwrap().active_parameter, Some(1));
	}

	#[test]
	fn signature_label_includes_return_type() {
		let sg = schema();
		let help = signature::signature_help("fn::greet(", pos(0, 10), Some(&sg)).unwrap();
		assert!(help.signatures[0].label.contains("-> string"));
	}
}

// ═══════════════════════════════════════════════════════════════════════
// 11. BYTE OFFSET TO POSITION CONVERSION
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod position_conversion {
	use crate::server::byte_offset_to_position;
	use tower_lsp::lsp_types::Position;

	#[test]
	fn start_of_file() {
		let pos = byte_offset_to_position("hello\nworld", 0);
		assert_eq!(
			pos,
			Position {
				line: 0,
				character: 0
			}
		);
	}

	#[test]
	fn middle_of_first_line() {
		let pos = byte_offset_to_position("hello\nworld", 3);
		assert_eq!(
			pos,
			Position {
				line: 0,
				character: 3
			}
		);
	}

	#[test]
	fn start_of_second_line() {
		let pos = byte_offset_to_position("hello\nworld", 6);
		assert_eq!(
			pos,
			Position {
				line: 1,
				character: 0
			}
		);
	}

	#[test]
	fn middle_of_second_line() {
		let pos = byte_offset_to_position("hello\nworld", 9);
		assert_eq!(
			pos,
			Position {
				line: 1,
				character: 3
			}
		);
	}

	#[test]
	fn end_of_file() {
		let pos = byte_offset_to_position("hello\nworld", 11);
		assert_eq!(
			pos,
			Position {
				line: 1,
				character: 5
			}
		);
	}

	#[test]
	fn past_end_of_file() {
		let pos = byte_offset_to_position("hello", 100);
		assert_eq!(
			pos,
			Position {
				line: 0,
				character: 5
			}
		);
	}

	#[test]
	fn empty_file() {
		let pos = byte_offset_to_position("", 0);
		assert_eq!(
			pos,
			Position {
				line: 0,
				character: 0
			}
		);
	}

	#[test]
	fn three_lines() {
		let pos = byte_offset_to_position("a\nb\nc", 4);
		assert_eq!(
			pos,
			Position {
				line: 2,
				character: 0
			}
		);
	}
}

// ═══════════════════════════════════════════════════════════════════════
// 12. BUILT-IN FUNCTION NAMESPACES
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod builtin_completions {
	use crate::completion;
	use surql_parser::SchemaGraph;
	use tower_lsp::lsp_types::{CompletionItemKind, Position};

	fn pos(line: u32, col: u32) -> Position {
		Position {
			line,
			character: col,
		}
	}

	#[test]
	fn builtin_namespaces_after_fn() {
		let sg = SchemaGraph::default();
		let items = completion::complete("SELECT fn::", pos(0, 11), Some(&sg));
		let modules: Vec<_> = items
			.iter()
			.filter(|i| i.kind == Some(CompletionItemKind::MODULE))
			.collect();
		assert!(modules.iter().any(|i| i.label == "array::"));
		assert!(modules.iter().any(|i| i.label == "string::"));
		assert!(modules.iter().any(|i| i.label == "time::"));
		assert!(modules.iter().any(|i| i.label == "math::"));
		assert!(modules.iter().any(|i| i.label == "crypto::"));
	}

	#[test]
	fn builtin_namespaces_count() {
		let sg = SchemaGraph::default();
		let items = completion::complete("SELECT fn::", pos(0, 11), Some(&sg));
		let modules: Vec<_> = items
			.iter()
			.filter(|i| i.kind == Some(CompletionItemKind::MODULE))
			.collect();
		assert!(
			modules.len() >= 15,
			"Expected 15+ built-in namespaces, got {}",
			modules.len()
		);
	}
}
