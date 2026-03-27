//! Tests for SchemaGraph — semantic model from SurrealQL definitions.

use surql_parser::SchemaGraph;

#[test]
fn basic_table_and_fields() {
	let sg = SchemaGraph::from_source(
		"
		DEFINE TABLE user SCHEMAFULL;
		DEFINE FIELD name ON user TYPE string;
		DEFINE FIELD email ON user TYPE string;
		DEFINE FIELD age ON user TYPE int;
	",
	)
	.unwrap();

	assert_eq!(sg.table_names().count(), 1);
	assert!(sg.table("user").is_some());
	assert_eq!(sg.fields_of("user").len(), 3);

	let name_field = sg
		.fields_of("user")
		.iter()
		.find(|f| f.name == "name")
		.unwrap();
	assert_eq!(name_field.kind.as_deref(), Some("string"));
}

#[test]
fn table_type_info() {
	let sg = SchemaGraph::from_source("DEFINE TABLE user SCHEMAFULL").unwrap();
	let t = sg.table("user").unwrap();
	assert!(t.full);
}

#[test]
fn schemaless_table() {
	let sg = SchemaGraph::from_source("DEFINE TABLE log SCHEMALESS").unwrap();
	let t = sg.table("log").unwrap();
	assert!(!t.full);
}

#[test]
fn indexes() {
	let sg = SchemaGraph::from_source(
		"
		DEFINE TABLE user SCHEMAFULL;
		DEFINE INDEX email_idx ON user FIELDS email UNIQUE;
	",
	)
	.unwrap();

	let indexes = sg.indexes_of("user");
	assert_eq!(indexes.len(), 1);
	assert_eq!(indexes[0].name, "email_idx");
	assert!(indexes[0].unique);
}

#[test]
fn functions() {
	let sg = SchemaGraph::from_source(
		"DEFINE FUNCTION fn::greet($name: string) -> string { RETURN 'Hello, ' + $name; }",
	)
	.unwrap();

	let f = sg.function("greet").unwrap();
	assert_eq!(f.args.len(), 1);
	assert_eq!(f.args[0].0, "$name");
	assert_eq!(f.args[0].1, "string");
	assert_eq!(f.returns.as_deref(), Some("string"));
}

#[test]
fn function_names() {
	let sg = SchemaGraph::from_source(
		"
		DEFINE FUNCTION fn::greet($name: string) { RETURN 'Hello, ' + $name; };
		DEFINE FUNCTION fn::add($a: int, $b: int) { RETURN $a + $b; };
	",
	)
	.unwrap();

	let mut names: Vec<&str> = sg.function_names().collect();
	names.sort();
	assert_eq!(names, vec!["add", "greet"]);
}

#[test]
fn record_links() {
	let sg = SchemaGraph::from_source(
		"
		DEFINE TABLE user SCHEMAFULL;
		DEFINE TABLE post SCHEMAFULL;
		DEFINE FIELD author ON post TYPE record<user>;
	",
	)
	.unwrap();

	let fields = sg.fields_of("post");
	let author = fields.iter().find(|f| f.name == "author").unwrap();
	assert_eq!(author.record_links, vec!["user"]);
}

#[test]
fn field_with_default() {
	let sg = SchemaGraph::from_source(
		"
		DEFINE TABLE user SCHEMAFULL;
		DEFINE FIELD created_at ON user TYPE datetime DEFAULT time::now();
	",
	)
	.unwrap();

	let fields = sg.fields_of("user");
	let created = fields.iter().find(|f| f.name == "created_at").unwrap();
	assert!(created.default.is_some());
}

#[test]
fn merge_graphs() {
	let mut sg1 = SchemaGraph::from_source(
		"
		DEFINE TABLE user SCHEMAFULL;
		DEFINE FIELD name ON user TYPE string;
	",
	)
	.unwrap();

	let sg2 = SchemaGraph::from_source(
		"
		DEFINE TABLE post SCHEMAFULL;
		DEFINE FIELD title ON post TYPE string;
	",
	)
	.unwrap();

	sg1.merge(sg2);
	assert_eq!(sg1.table_names().count(), 2);
	assert!(sg1.table("user").is_some());
	assert!(sg1.table("post").is_some());
}

#[test]
fn merge_adds_fields_to_existing_table() {
	let mut sg1 = SchemaGraph::from_source(
		"
		DEFINE TABLE user SCHEMAFULL;
		DEFINE FIELD name ON user TYPE string;
	",
	)
	.unwrap();

	let sg2 = SchemaGraph::from_source("DEFINE FIELD age ON user TYPE int").unwrap();

	sg1.merge(sg2);
	assert_eq!(sg1.fields_of("user").len(), 2);
}

#[test]
fn implicit_table_from_field() {
	let sg = SchemaGraph::from_source("DEFINE FIELD name ON user TYPE string").unwrap();
	assert!(sg.table("user").is_some());
	assert_eq!(sg.fields_of("user").len(), 1);
}

#[test]
fn relation_table() {
	let sg = SchemaGraph::from_source(
		"DEFINE TABLE follows TYPE RELATION FROM user TO user ENFORCED SCHEMAFULL",
	)
	.unwrap();
	let t = sg.table("follows").unwrap();
	assert!(t.full);
}

#[test]
fn empty_schema() {
	let sg = SchemaGraph::from_source("SELECT * FROM user").unwrap();
	assert_eq!(sg.table_names().count(), 0);
	assert_eq!(sg.function_names().count(), 0);
}

#[test]
fn events() {
	let sg = SchemaGraph::from_source(
		"
		DEFINE TABLE user SCHEMAFULL;
		DEFINE EVENT user_created ON user WHEN $event = 'CREATE' THEN { CREATE audit SET action = 'created' };
	",
	)
	.unwrap();

	let events = sg.events_of("user");
	assert_eq!(events.len(), 1);
	assert_eq!(events[0].name, "user_created");
}

#[test]
fn should_extract_table_comment() {
	let sg = SchemaGraph::from_source(
		"DEFINE TABLE user SCHEMAFULL COMMENT 'Main user table'; \
		 DEFINE FIELD name ON user TYPE string COMMENT 'Full name';",
	)
	.unwrap();
	assert_eq!(
		sg.table("user").unwrap().comment.as_deref(),
		Some("Main user table")
	);
	assert_eq!(
		sg.fields_of("user")[0].comment.as_deref(),
		Some("Full name")
	);
}

#[test]
fn should_extract_function_comment() {
	let sg = SchemaGraph::from_source(
		"DEFINE FUNCTION fn::greet($name: string) -> string \
		 { RETURN 'Hello, ' + $name; } COMMENT 'Greeting function'",
	)
	.unwrap();
	assert_eq!(
		sg.function("greet").unwrap().comment.as_deref(),
		Some("Greeting function")
	);
}

#[test]
fn should_extract_index_comment() {
	let sg = SchemaGraph::from_source(
		"DEFINE TABLE user SCHEMAFULL; \
		 DEFINE INDEX email_idx ON user FIELDS email UNIQUE COMMENT 'Unique email constraint';",
	)
	.unwrap();
	assert_eq!(
		sg.indexes_of("user")[0].comment.as_deref(),
		Some("Unique email constraint")
	);
}

#[test]
fn should_extract_event_comment() {
	let sg = SchemaGraph::from_source(
		"DEFINE TABLE user SCHEMAFULL; \
		 DEFINE EVENT user_created ON user WHEN $event = 'CREATE' \
		 THEN { CREATE audit SET action = 'created' } COMMENT 'Audit trail';",
	)
	.unwrap();
	assert_eq!(
		sg.events_of("user")[0].comment.as_deref(),
		Some("Audit trail")
	);
}

#[test]
fn should_return_none_when_no_comment() {
	let sg = SchemaGraph::from_source(
		"DEFINE TABLE user SCHEMAFULL; DEFINE FIELD name ON user TYPE string;",
	)
	.unwrap();
	assert!(sg.table("user").unwrap().comment.is_none());
	assert!(sg.fields_of("user")[0].comment.is_none());
}

#[test]
fn find_field_uses_index() {
	let table_count = 100;
	let fields_per_table = 10;

	let mut stmts = String::new();
	for t in 0..table_count {
		stmts.push_str(&format!("DEFINE TABLE tbl_{t} SCHEMAFULL;\n"));
		for f in 0..fields_per_table {
			stmts.push_str(&format!("DEFINE FIELD fld_{f} ON tbl_{t} TYPE string;\n"));
		}
	}

	let sg = SchemaGraph::from_source(&stmts).unwrap();

	assert_eq!(sg.table_names().count(), table_count);

	let results = sg.find_field("fld_0");
	assert_eq!(results.len(), table_count);
	for (table_name, field) in &results {
		assert!(table_name.starts_with("tbl_"));
		assert_eq!(field.name, "fld_0");
	}

	let results = sg.find_field("fld_9");
	assert_eq!(results.len(), table_count);

	let results = sg.find_field("nonexistent");
	assert!(results.is_empty());
}

#[test]
fn find_field_index_maintained_after_merge() {
	let mut sg1 = SchemaGraph::from_source(
		"DEFINE TABLE user SCHEMAFULL; DEFINE FIELD name ON user TYPE string;",
	)
	.unwrap();

	let sg2 = SchemaGraph::from_source(
		"DEFINE TABLE post SCHEMAFULL; DEFINE FIELD name ON post TYPE string;",
	)
	.unwrap();

	sg1.merge(sg2);

	let results = sg1.find_field("name");
	assert_eq!(results.len(), 2);

	let table_names: Vec<&str> = results.iter().map(|(t, _)| *t).collect();
	assert!(table_names.contains(&"user"));
	assert!(table_names.contains(&"post"));
}

#[test]
fn find_field_index_deduplicates_on_merge() {
	let mut sg1 = SchemaGraph::from_source(
		"DEFINE TABLE user SCHEMAFULL; DEFINE FIELD name ON user TYPE string;",
	)
	.unwrap();

	let sg2 = SchemaGraph::from_source("DEFINE FIELD name ON user TYPE string;").unwrap();

	sg1.merge(sg2);

	let results = sg1.find_field("name");
	assert_eq!(results.len(), 1);
}

#[test]
fn should_track_ns_db_from_use_statement() {
	let sg = SchemaGraph::from_source(
		"
		USE NS production DB main;
		DEFINE TABLE user SCHEMAFULL;
		DEFINE FIELD name ON user TYPE string;
	",
	)
	.unwrap();

	let table = sg.table("user").unwrap();
	assert_eq!(table.ns.as_deref(), Some("production"));
	assert_eq!(table.db.as_deref(), Some("main"));
}

#[test]
fn should_scope_tables_by_ns_db() {
	let mut sg =
		SchemaGraph::from_source("USE NS prod DB main; DEFINE TABLE user SCHEMAFULL;").unwrap();
	let sg2 =
		SchemaGraph::from_source("USE NS test DB fixtures; DEFINE TABLE mock_user SCHEMAFULL;")
			.unwrap();
	sg.merge(sg2);

	let prod = sg.scoped(Some("prod"), Some("main"));
	assert!(prod.table("user").is_some());
	assert!(prod.table("mock_user").is_none());

	let test = sg.scoped(Some("test"), Some("fixtures"));
	assert!(test.table("mock_user").is_some());
	assert!(test.table("user").is_none());
}

#[test]
fn should_include_unscoped_tables_in_any_scope() {
	let mut sg = SchemaGraph::from_source("DEFINE TABLE global SCHEMAFULL;").unwrap();
	let sg2 =
		SchemaGraph::from_source("USE NS prod DB main; DEFINE TABLE scoped SCHEMAFULL;").unwrap();
	sg.merge(sg2);

	let prod = sg.scoped(Some("prod"), Some("main"));
	assert!(
		prod.table("global").is_some(),
		"unscoped tables visible everywhere"
	);
	assert!(prod.table("scoped").is_some());
}

#[test]
fn should_default_ns_db_to_none() {
	let sg = SchemaGraph::from_source("DEFINE TABLE user SCHEMAFULL;").unwrap();
	let table = sg.table("user").unwrap();
	assert_eq!(table.ns, None);
	assert_eq!(table.db, None);
}

#[test]
fn should_build_from_single_file() {
	let dir = tempfile::tempdir().unwrap();
	let file = dir.path().join("schema.surql");
	std::fs::write(
		&file,
		"DEFINE TABLE user SCHEMAFULL;\nDEFINE FIELD name ON user TYPE string;\n",
	)
	.unwrap();

	let sg = SchemaGraph::from_single_file(&file).unwrap();
	assert_eq!(sg.table_names().count(), 1);
	assert!(sg.table("user").unwrap().full);
	assert_eq!(sg.fields_of("user").len(), 1);
	assert_eq!(sg.fields_of("user")[0].name, "name");
}

#[test]
fn should_return_none_for_nonexistent_file() {
	let result = SchemaGraph::from_single_file(std::path::Path::new("/nonexistent/missing.surql"));
	assert!(result.is_none());
}

#[test]
fn should_build_per_file_from_directory() {
	let dir = tempfile::tempdir().unwrap();
	std::fs::write(
		dir.path().join("users.surql"),
		"DEFINE TABLE user SCHEMAFULL;\nDEFINE FIELD name ON user TYPE string;\n",
	)
	.unwrap();
	std::fs::write(
		dir.path().join("posts.surql"),
		"DEFINE TABLE post SCHEMAFULL;\nDEFINE FIELD title ON post TYPE string;\n",
	)
	.unwrap();

	let per_file = SchemaGraph::from_files_per_file(dir.path()).unwrap();
	assert_eq!(per_file.len(), 2);

	let has_user_file = per_file.values().any(|sg| sg.table("user").is_some());
	let has_post_file = per_file.values().any(|sg| sg.table("post").is_some());
	assert!(has_user_file);
	assert!(has_post_file);

	// Each file should have exactly one table
	for sg in per_file.values() {
		assert_eq!(sg.table_names().count(), 1);
	}
}

#[test]
fn should_merge_per_file_schemas_into_complete_graph() {
	let dir = tempfile::tempdir().unwrap();
	std::fs::write(
		dir.path().join("users.surql"),
		"DEFINE TABLE user SCHEMAFULL;\nDEFINE FIELD name ON user TYPE string;\n",
	)
	.unwrap();
	std::fs::write(
		dir.path().join("posts.surql"),
		"DEFINE TABLE post SCHEMAFULL;\nDEFINE FIELD author ON post TYPE record<user>;\n",
	)
	.unwrap();

	let per_file = SchemaGraph::from_files_per_file(dir.path()).unwrap();
	let mut merged = SchemaGraph::default();
	for sg in per_file.values() {
		merged.merge(sg.clone());
	}

	assert_eq!(merged.table_names().count(), 2);
	assert!(merged.table("user").is_some());
	assert!(merged.table("post").is_some());
	assert_eq!(merged.fields_of("post")[0].record_links, vec!["user"]);
}

#[test]
fn should_produce_same_result_as_from_files() {
	let dir = tempfile::tempdir().unwrap();
	std::fs::write(
		dir.path().join("users.surql"),
		"DEFINE TABLE user SCHEMAFULL;\nDEFINE FIELD name ON user TYPE string;\n",
	)
	.unwrap();
	std::fs::write(
		dir.path().join("posts.surql"),
		"DEFINE TABLE post SCHEMAFULL;\nDEFINE FIELD title ON post TYPE string;\n",
	)
	.unwrap();

	let from_files = SchemaGraph::from_files(dir.path()).unwrap();

	let per_file = SchemaGraph::from_files_per_file(dir.path()).unwrap();
	let mut merged = SchemaGraph::default();
	for sg in per_file.values() {
		merged.merge(sg.clone());
	}

	// Same tables
	let mut from_files_tables: Vec<&str> = from_files.table_names().collect();
	from_files_tables.sort();
	let mut merged_tables: Vec<&str> = merged.table_names().collect();
	merged_tables.sort();
	assert_eq!(from_files_tables, merged_tables);

	// Same fields per table
	for table in &from_files_tables {
		let from_files_fields: Vec<&str> = from_files
			.fields_of(table)
			.iter()
			.map(|f| f.name.as_str())
			.collect();
		let merged_fields: Vec<&str> = merged
			.fields_of(table)
			.iter()
			.map(|f| f.name.as_str())
			.collect();
		assert_eq!(
			from_files_fields, merged_fields,
			"fields mismatch for table {table}"
		);
	}
}

#[test]
fn should_handle_empty_directory() {
	let dir = tempfile::tempdir().unwrap();
	let per_file = SchemaGraph::from_files_per_file(dir.path()).unwrap();
	assert!(per_file.is_empty());
}

#[test]
fn should_skip_unparsable_files() {
	let dir = tempfile::tempdir().unwrap();
	std::fs::write(
		dir.path().join("good.surql"),
		"DEFINE TABLE user SCHEMAFULL;\n",
	)
	.unwrap();
	std::fs::write(dir.path().join("not_surql.txt"), "just a text file").unwrap();

	let per_file = SchemaGraph::from_files_per_file(dir.path()).unwrap();
	assert_eq!(per_file.len(), 1);
}

// ─── Graph Traversal Tests ───

fn build_chain_graph() -> SchemaGraph {
	// A -> B -> C (linear chain)
	SchemaGraph::from_source(
		"
		DEFINE TABLE a SCHEMAFULL;
		DEFINE TABLE b SCHEMAFULL;
		DEFINE TABLE c SCHEMAFULL;
		DEFINE FIELD ref_b ON a TYPE record<b>;
		DEFINE FIELD ref_c ON b TYPE record<c>;
	",
	)
	.unwrap()
}

fn build_cycle_graph() -> SchemaGraph {
	// A -> B -> A (cycle)
	SchemaGraph::from_source(
		"
		DEFINE TABLE a SCHEMAFULL;
		DEFINE TABLE b SCHEMAFULL;
		DEFINE FIELD ref_b ON a TYPE record<b>;
		DEFINE FIELD ref_a ON b TYPE record<a>;
	",
	)
	.unwrap()
}

fn build_diamond_graph() -> SchemaGraph {
	// A -> B, A -> C, B -> D, C -> D (diamond)
	SchemaGraph::from_source(
		"
		DEFINE TABLE a SCHEMAFULL;
		DEFINE TABLE b SCHEMAFULL;
		DEFINE TABLE c SCHEMAFULL;
		DEFINE TABLE d SCHEMAFULL;
		DEFINE FIELD ref_b ON a TYPE record<b>;
		DEFINE FIELD ref_c ON a TYPE record<c>;
		DEFINE FIELD ref_d ON b TYPE record<d>;
		DEFINE FIELD ref_d ON c TYPE record<d>;
	",
	)
	.unwrap()
}

fn build_self_ref_graph() -> SchemaGraph {
	// A -> A (self-reference)
	SchemaGraph::from_source(
		"
		DEFINE TABLE a SCHEMAFULL;
		DEFINE FIELD parent ON a TYPE record<a>;
	",
	)
	.unwrap()
}

#[test]
fn should_traverse_linear_chain() {
	let sg = build_chain_graph();
	let reachable = sg.tables_reachable_from("a", 10);

	assert_eq!(reachable.len(), 2);
	assert_eq!(reachable[0].0, "b");
	assert_eq!(reachable[0].1, 1);
	assert_eq!(reachable[0].2, vec!["a.ref_b"]);

	assert_eq!(reachable[1].0, "c");
	assert_eq!(reachable[1].1, 2);
	assert_eq!(reachable[1].2, vec!["a.ref_b", "b.ref_c"]);
}

#[test]
fn should_traverse_with_max_depth_limit() {
	let sg = build_chain_graph();
	let reachable = sg.tables_reachable_from("a", 1);

	assert_eq!(reachable.len(), 1);
	assert_eq!(reachable[0].0, "b");
}

#[test]
fn should_traverse_cycle_without_infinite_loop() {
	let sg = build_cycle_graph();
	let reachable = sg.tables_reachable_from("a", 10);

	assert_eq!(reachable.len(), 1, "cycle should not cause duplicates");
	assert_eq!(reachable[0].0, "b");
	assert_eq!(reachable[0].1, 1);
}

#[test]
fn should_traverse_diamond_without_duplicates() {
	let sg = build_diamond_graph();
	let reachable = sg.tables_reachable_from("a", 10);

	// b, c at depth 1; d at depth 2 (only once)
	assert_eq!(reachable.len(), 3);

	let names: Vec<&str> = reachable.iter().map(|(n, _, _)| n.as_str()).collect();
	assert!(names.contains(&"b"));
	assert!(names.contains(&"c"));
	assert!(names.contains(&"d"));

	// d should appear exactly once
	assert_eq!(names.iter().filter(|&&n| n == "d").count(), 1);
}

#[test]
fn should_traverse_self_reference() {
	let sg = build_self_ref_graph();
	let reachable = sg.tables_reachable_from("a", 10);
	// Self-ref: 'a' is the start, so it's already visited, no results
	assert!(reachable.is_empty());
}

#[test]
fn should_traverse_from_nonexistent_table() {
	let sg = build_chain_graph();
	let reachable = sg.tables_reachable_from("nonexistent", 10);
	assert!(reachable.is_empty());
}

#[test]
fn should_traverse_from_isolated_table() {
	let sg = SchemaGraph::from_source(
		"
		DEFINE TABLE isolated SCHEMAFULL;
		DEFINE FIELD name ON isolated TYPE string;
		DEFINE TABLE other SCHEMAFULL;
	",
	)
	.unwrap();
	let reachable = sg.tables_reachable_from("isolated", 10);
	assert!(reachable.is_empty());
}

#[test]
fn should_traverse_zero_depth() {
	let sg = build_chain_graph();
	let reachable = sg.tables_reachable_from("a", 0);
	assert!(reachable.is_empty());
}

#[test]
fn should_find_reverse_dependencies() {
	let sg = build_chain_graph();

	let refs = sg.tables_referencing("b");
	assert_eq!(refs.len(), 1);
	assert_eq!(refs[0], ("a".to_string(), "ref_b".to_string()));

	let refs = sg.tables_referencing("c");
	assert_eq!(refs.len(), 1);
	assert_eq!(refs[0], ("b".to_string(), "ref_c".to_string()));

	let refs = sg.tables_referencing("a");
	assert!(refs.is_empty(), "nothing references 'a' in a chain");
}

#[test]
fn should_find_reverse_deps_in_diamond() {
	let sg = build_diamond_graph();

	let refs = sg.tables_referencing("d");
	assert_eq!(refs.len(), 2);
	let ref_tables: Vec<&str> = refs.iter().map(|(t, _)| t.as_str()).collect();
	assert!(ref_tables.contains(&"b"));
	assert!(ref_tables.contains(&"c"));
}

#[test]
fn should_find_reverse_deps_in_cycle() {
	let sg = build_cycle_graph();

	let refs = sg.tables_referencing("a");
	assert_eq!(refs.len(), 1);
	assert_eq!(refs[0], ("b".to_string(), "ref_a".to_string()));
}

#[test]
fn should_find_reverse_deps_for_nonexistent() {
	let sg = build_chain_graph();
	let refs = sg.tables_referencing("nonexistent");
	assert!(refs.is_empty());
}

#[test]
fn should_find_reverse_deps_for_self_reference() {
	let sg = build_self_ref_graph();
	let refs = sg.tables_referencing("a");
	assert_eq!(refs.len(), 1);
	assert_eq!(refs[0], ("a".to_string(), "parent".to_string()));
}

#[test]
fn should_find_siblings_in_diamond() {
	let sg = build_diamond_graph();

	// b and c both link to d
	let siblings = sg.siblings_of("b");
	assert!(!siblings.is_empty());
	let has_c_sibling = siblings.iter().any(|(s, t, _)| s == "c" && t == "d");
	assert!(has_c_sibling, "c should be a sibling of b (both link to d)");
}

#[test]
fn should_find_no_siblings_for_isolated_table() {
	let sg = SchemaGraph::from_source(
		"
		DEFINE TABLE isolated SCHEMAFULL;
		DEFINE FIELD name ON isolated TYPE string;
	",
	)
	.unwrap();
	let siblings = sg.siblings_of("isolated");
	assert!(siblings.is_empty());
}

#[test]
fn should_find_no_siblings_for_nonexistent_table() {
	let sg = build_chain_graph();
	let siblings = sg.siblings_of("nonexistent");
	assert!(siblings.is_empty());
}

#[test]
fn should_build_dependency_tree_for_chain() {
	let sg = build_chain_graph();
	let tree = sg.dependency_tree("a", 10);

	assert_eq!(tree.table, "a");
	assert!(!tree.is_cycle);
	assert!(tree.field.is_none());
	assert_eq!(tree.children.len(), 1);

	let b_node = &tree.children[0];
	assert_eq!(b_node.table, "b");
	assert_eq!(b_node.field.as_deref(), Some("ref_b"));
	assert!(!b_node.is_cycle);
	assert_eq!(b_node.children.len(), 1);

	let c_node = &b_node.children[0];
	assert_eq!(c_node.table, "c");
	assert_eq!(c_node.field.as_deref(), Some("ref_c"));
	assert!(!c_node.is_cycle);
	assert!(c_node.children.is_empty());
}

#[test]
fn should_build_dependency_tree_with_cycle_detection() {
	let sg = build_cycle_graph();
	let tree = sg.dependency_tree("a", 10);

	assert_eq!(tree.table, "a");
	assert!(!tree.is_cycle);
	assert_eq!(tree.children.len(), 1);

	let b_node = &tree.children[0];
	assert_eq!(b_node.table, "b");
	assert!(!b_node.is_cycle);
	assert_eq!(b_node.children.len(), 1);

	let cycle_a = &b_node.children[0];
	assert_eq!(cycle_a.table, "a");
	assert!(cycle_a.is_cycle, "should detect cycle back to 'a'");
	assert!(cycle_a.children.is_empty());
}

#[test]
fn should_build_dependency_tree_for_diamond() {
	let sg = build_diamond_graph();
	let tree = sg.dependency_tree("a", 10);

	assert_eq!(tree.table, "a");
	assert_eq!(tree.children.len(), 2);

	// Both branches should reach d independently (not a cycle, since we
	// use per-path visited tracking in dependency_tree)
	let b_node = tree.children.iter().find(|n| n.table == "b").unwrap();
	let c_node = tree.children.iter().find(|n| n.table == "c").unwrap();

	assert_eq!(b_node.children.len(), 1);
	assert_eq!(b_node.children[0].table, "d");

	assert_eq!(c_node.children.len(), 1);
	assert_eq!(c_node.children[0].table, "d");
}

#[test]
fn should_build_dependency_tree_self_reference() {
	let sg = build_self_ref_graph();
	let tree = sg.dependency_tree("a", 10);

	assert_eq!(tree.table, "a");
	assert_eq!(tree.children.len(), 1);

	let cycle_node = &tree.children[0];
	assert_eq!(cycle_node.table, "a");
	assert!(cycle_node.is_cycle);
}

#[test]
fn should_build_dependency_tree_respects_max_depth() {
	let sg = build_chain_graph();
	let tree = sg.dependency_tree("a", 1);

	assert_eq!(tree.children.len(), 1);
	let b_node = &tree.children[0];
	assert_eq!(b_node.table, "b");
	assert!(
		b_node.children.is_empty(),
		"depth=1 should stop at first hop"
	);
}

#[test]
fn should_build_graph_tree_markdown() {
	let sg = build_chain_graph();
	let md = sg.build_graph_tree_markdown();

	assert!(md.contains("# Schema Graph"));
	assert!(md.contains("3 tables"));
	assert!(md.contains("## a"));
	assert!(md.contains("Depends on"));
	assert!(md.contains("[b]"));
	assert!(md.contains("[c]"));
	assert!(md.contains("Referenced by"));
}

#[test]
fn should_build_graph_tree_markdown_empty() {
	let sg = SchemaGraph::default();
	let md = sg.build_graph_tree_markdown();
	assert!(md.contains("No tables defined"));
}

#[test]
fn should_build_graph_tree_markdown_isolated_tables_omitted() {
	let sg = SchemaGraph::from_source(
		"
		DEFINE TABLE a SCHEMAFULL;
		DEFINE FIELD name ON a TYPE string;
		DEFINE TABLE b SCHEMAFULL;
		DEFINE TABLE c SCHEMAFULL;
		DEFINE FIELD ref_a ON c TYPE record<a>;
	",
	)
	.unwrap();
	let md = sg.build_graph_tree_markdown();
	// 'b' has no links at all, should not get its own section
	assert!(!md.contains("## b\n"));
	// 'a' is referenced by c, should get a section
	assert!(md.contains("## a"));
	assert!(md.contains("## c"));
}
