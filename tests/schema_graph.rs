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
fn should_default_ns_db_to_none() {
	let sg = SchemaGraph::from_source("DEFINE TABLE user SCHEMAFULL;").unwrap();
	let table = sg.table("user").unwrap();
	assert_eq!(table.ns, None);
	assert_eq!(table.db, None);
}
