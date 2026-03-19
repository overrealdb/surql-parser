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

	assert_eq!(sg.table_names().len(), 1);
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

	let mut names = sg.function_names();
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
	assert_eq!(sg1.table_names().len(), 2);
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
	assert!(sg.table_names().is_empty());
	assert!(sg.function_names().is_empty());
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
