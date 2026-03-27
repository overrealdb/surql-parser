use surql_parser::SchemaGraph;
use surql_parser::diff::{FieldTypeChange, SchemaDiff, TableChange, compare_schemas};

#[test]
fn should_detect_no_changes_for_identical_schemas() {
	let source = "
		DEFINE TABLE user SCHEMAFULL;
		DEFINE FIELD name ON user TYPE string;
		DEFINE INDEX email_idx ON user FIELDS email UNIQUE;
		DEFINE FUNCTION fn::greet($name: string) -> string { RETURN 'Hello, ' + $name; };
	";
	let before = SchemaGraph::from_source(source).unwrap();
	let after = SchemaGraph::from_source(source).unwrap();

	let diff = compare_schemas(&before, &after);
	assert!(diff.is_empty());
}

#[test]
fn should_detect_added_table() {
	let before = SchemaGraph::from_source("DEFINE TABLE user SCHEMAFULL;").unwrap();
	let after =
		SchemaGraph::from_source("DEFINE TABLE user SCHEMAFULL; DEFINE TABLE post SCHEMALESS;")
			.unwrap();

	let diff = compare_schemas(&before, &after);
	assert_eq!(diff.added_tables, vec!["post"]);
	assert!(diff.removed_tables.is_empty());
}

#[test]
fn should_detect_removed_table() {
	let before =
		SchemaGraph::from_source("DEFINE TABLE user SCHEMAFULL; DEFINE TABLE post SCHEMALESS;")
			.unwrap();
	let after = SchemaGraph::from_source("DEFINE TABLE user SCHEMAFULL;").unwrap();

	let diff = compare_schemas(&before, &after);
	assert!(diff.added_tables.is_empty());
	assert_eq!(diff.removed_tables, vec!["post"]);
}

#[test]
fn should_detect_added_field() {
	let before = SchemaGraph::from_source(
		"DEFINE TABLE user SCHEMAFULL; DEFINE FIELD name ON user TYPE string;",
	)
	.unwrap();
	let after = SchemaGraph::from_source(
		"DEFINE TABLE user SCHEMAFULL; DEFINE FIELD name ON user TYPE string; DEFINE FIELD email ON user TYPE string;",
	)
	.unwrap();

	let diff = compare_schemas(&before, &after);
	assert_eq!(
		diff.added_fields,
		vec![("user".to_string(), "email".to_string())]
	);
	assert!(diff.removed_fields.is_empty());
}

#[test]
fn should_detect_removed_field() {
	let before = SchemaGraph::from_source(
		"DEFINE TABLE user SCHEMAFULL; DEFINE FIELD name ON user TYPE string; DEFINE FIELD email ON user TYPE string;",
	)
	.unwrap();
	let after = SchemaGraph::from_source(
		"DEFINE TABLE user SCHEMAFULL; DEFINE FIELD name ON user TYPE string;",
	)
	.unwrap();

	let diff = compare_schemas(&before, &after);
	assert!(diff.added_fields.is_empty());
	assert_eq!(
		diff.removed_fields,
		vec![("user".to_string(), "email".to_string())]
	);
}

#[test]
fn should_detect_added_index() {
	let before = SchemaGraph::from_source("DEFINE TABLE user SCHEMAFULL;").unwrap();
	let after = SchemaGraph::from_source(
		"DEFINE TABLE user SCHEMAFULL; DEFINE INDEX email_idx ON user FIELDS email UNIQUE;",
	)
	.unwrap();

	let diff = compare_schemas(&before, &after);
	assert_eq!(
		diff.added_indexes,
		vec![("user".to_string(), "email_idx".to_string())]
	);
}

#[test]
fn should_detect_removed_index() {
	let before = SchemaGraph::from_source(
		"DEFINE TABLE user SCHEMAFULL; DEFINE INDEX email_idx ON user FIELDS email UNIQUE;",
	)
	.unwrap();
	let after = SchemaGraph::from_source("DEFINE TABLE user SCHEMAFULL;").unwrap();

	let diff = compare_schemas(&before, &after);
	assert_eq!(
		diff.removed_indexes,
		vec![("user".to_string(), "email_idx".to_string())]
	);
}

#[test]
fn should_detect_added_function() {
	let before = SchemaGraph::from_source("DEFINE TABLE user SCHEMAFULL;").unwrap();
	let after = SchemaGraph::from_source(
		"DEFINE TABLE user SCHEMAFULL; DEFINE FUNCTION fn::greet($name: string) -> string { RETURN 'Hello, ' + $name; };",
	)
	.unwrap();

	let diff = compare_schemas(&before, &after);
	assert_eq!(diff.added_functions, vec!["greet"]);
}

#[test]
fn should_detect_removed_function() {
	let before = SchemaGraph::from_source(
		"DEFINE TABLE user SCHEMAFULL; DEFINE FUNCTION fn::greet($name: string) -> string { RETURN 'Hello, ' + $name; };",
	)
	.unwrap();
	let after = SchemaGraph::from_source("DEFINE TABLE user SCHEMAFULL;").unwrap();

	let diff = compare_schemas(&before, &after);
	assert_eq!(diff.removed_functions, vec!["greet"]);
}

#[test]
fn should_detect_schemafull_to_schemaless_change() {
	let before = SchemaGraph::from_source("DEFINE TABLE user SCHEMAFULL;").unwrap();
	let after = SchemaGraph::from_source("DEFINE TABLE user SCHEMALESS;").unwrap();

	let diff = compare_schemas(&before, &after);
	assert_eq!(diff.changed_tables.len(), 1);
	assert_eq!(diff.changed_tables[0].name, "user");
	assert!(diff.changed_tables[0].before_full);
	assert!(!diff.changed_tables[0].after_full);
}

#[test]
fn should_detect_schemaless_to_schemafull_change() {
	let before = SchemaGraph::from_source("DEFINE TABLE user SCHEMALESS;").unwrap();
	let after = SchemaGraph::from_source("DEFINE TABLE user SCHEMAFULL;").unwrap();

	let diff = compare_schemas(&before, &after);
	assert_eq!(diff.changed_tables.len(), 1);
	assert!(!diff.changed_tables[0].before_full);
	assert!(diff.changed_tables[0].after_full);
}

#[test]
fn should_detect_multiple_changes_across_tables() {
	let before = SchemaGraph::from_source(
		"\
DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD name ON user TYPE string;
DEFINE TABLE session SCHEMALESS;
DEFINE FUNCTION fn::old() { RETURN 1; };
",
	)
	.unwrap();

	let after = SchemaGraph::from_source(
		"\
DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD name ON user TYPE string;
DEFINE FIELD email ON user TYPE string;
DEFINE TABLE post SCHEMALESS;
DEFINE FUNCTION fn::greet($name: string) { RETURN 'Hello, ' + $name; };
",
	)
	.unwrap();

	let diff = compare_schemas(&before, &after);
	assert_eq!(diff.added_tables, vec!["post"]);
	assert_eq!(diff.removed_tables, vec!["session"]);
	assert_eq!(
		diff.added_fields,
		vec![("user".to_string(), "email".to_string())]
	);
	assert_eq!(diff.added_functions, vec!["greet"]);
	assert_eq!(diff.removed_functions, vec!["old"]);
}

#[test]
fn should_produce_empty_diff_from_two_empty_schemas() {
	let before = SchemaGraph::default();
	let after = SchemaGraph::default();

	let diff = compare_schemas(&before, &after);
	assert!(diff.is_empty());
}

#[test]
fn should_treat_all_definitions_as_added_when_before_is_empty() {
	let before = SchemaGraph::default();
	let after = SchemaGraph::from_source(
		"\
DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD name ON user TYPE string;
DEFINE INDEX email_idx ON user FIELDS email UNIQUE;
DEFINE FUNCTION fn::greet($name: string) { RETURN 'Hello, ' + $name; };
",
	)
	.unwrap();

	let diff = compare_schemas(&before, &after);
	assert_eq!(diff.added_tables, vec!["user"]);
	assert_eq!(
		diff.added_fields,
		vec![("user".to_string(), "name".to_string())]
	);
	assert_eq!(
		diff.added_indexes,
		vec![("user".to_string(), "email_idx".to_string())]
	);
	assert_eq!(diff.added_functions, vec!["greet"]);
}

#[test]
fn should_treat_all_definitions_as_removed_when_after_is_empty() {
	let before = SchemaGraph::from_source(
		"\
DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD name ON user TYPE string;
DEFINE FUNCTION fn::greet($name: string) { RETURN 'Hello, ' + $name; };
",
	)
	.unwrap();
	let after = SchemaGraph::default();

	let diff = compare_schemas(&before, &after);
	assert_eq!(diff.removed_tables, vec!["user"]);
	assert_eq!(
		diff.removed_fields,
		vec![("user".to_string(), "name".to_string())]
	);
	assert_eq!(diff.removed_functions, vec!["greet"]);
}

#[test]
fn should_detect_added_event() {
	let before = SchemaGraph::from_source("DEFINE TABLE user SCHEMAFULL;").unwrap();
	let after = SchemaGraph::from_source(
		"DEFINE TABLE user SCHEMAFULL; DEFINE EVENT user_created ON user WHEN $event = 'CREATE' THEN {};",
	)
	.unwrap();

	let diff = compare_schemas(&before, &after);
	assert_eq!(
		diff.added_events,
		vec![("user".to_string(), "user_created".to_string())]
	);
}

#[test]
fn should_detect_removed_event() {
	let before = SchemaGraph::from_source(
		"DEFINE TABLE user SCHEMAFULL; DEFINE EVENT user_created ON user WHEN $event = 'CREATE' THEN {};",
	)
	.unwrap();
	let after = SchemaGraph::from_source("DEFINE TABLE user SCHEMAFULL;").unwrap();

	let diff = compare_schemas(&before, &after);
	assert_eq!(
		diff.removed_events,
		vec![("user".to_string(), "user_created".to_string())]
	);
}

#[test]
fn should_format_display_with_all_change_types() {
	let diff = SchemaDiff {
		added_tables: vec!["post".to_string()],
		removed_tables: vec!["session".to_string()],
		changed_tables: vec![TableChange {
			name: "user".to_string(),
			before_full: true,
			after_full: false,
		}],
		added_fields: vec![("user".to_string(), "email".to_string())],
		removed_fields: vec![("user".to_string(), "old_field".to_string())],
		changed_fields: vec![FieldTypeChange {
			table: "user".to_string(),
			field: "name".to_string(),
			before_type: "string".to_string(),
			after_type: "int".to_string(),
		}],
		added_indexes: vec![("user".to_string(), "email_idx".to_string())],
		removed_indexes: vec![("user".to_string(), "name_idx".to_string())],
		added_events: vec![("user".to_string(), "on_create".to_string())],
		removed_events: vec![("user".to_string(), "on_delete".to_string())],
		added_functions: vec!["greet".to_string()],
		removed_functions: vec!["old_fn".to_string()],
	};

	let output = format!("{diff}");
	assert!(output.contains("+ TABLE post"));
	assert!(output.contains("- TABLE session"));
	assert!(output.contains("~ TABLE user: SCHEMAFULL -> SCHEMALESS"));
	assert!(output.contains("+ FIELD email ON user"));
	assert!(output.contains("- FIELD old_field ON user"));
	assert!(output.contains("~ FIELD name ON user: string -> int"));
	assert!(output.contains("+ INDEX email_idx ON user"));
	assert!(output.contains("- INDEX name_idx ON user"));
	assert!(output.contains("+ EVENT on_create ON user"));
	assert!(output.contains("- EVENT on_delete ON user"));
	assert!(output.contains("+ FUNCTION fn::greet"));
	assert!(output.contains("- FUNCTION fn::old_fn"));
}

#[test]
fn should_display_no_changes_for_empty_diff() {
	let diff = SchemaDiff::default();
	assert_eq!(format!("{diff}"), "No schema changes.");
}

#[test]
fn should_detect_field_type_change() {
	let before = SchemaGraph::from_source(
		"DEFINE TABLE user SCHEMAFULL; DEFINE FIELD name ON user TYPE string;",
	)
	.unwrap();
	let after = SchemaGraph::from_source(
		"DEFINE TABLE user SCHEMAFULL; DEFINE FIELD name ON user TYPE int;",
	)
	.unwrap();

	let diff = compare_schemas(&before, &after);
	assert!(diff.added_fields.is_empty());
	assert!(diff.removed_fields.is_empty());
	assert_eq!(diff.changed_fields.len(), 1);
	assert_eq!(diff.changed_fields[0].table, "user");
	assert_eq!(diff.changed_fields[0].field, "name");
	assert!(diff.changed_fields[0].before_type.contains("string"));
	assert!(diff.changed_fields[0].after_type.contains("int"));
}

#[test]
fn should_not_report_field_type_change_when_type_unchanged() {
	let source = "DEFINE TABLE user SCHEMAFULL; DEFINE FIELD name ON user TYPE string;";
	let before = SchemaGraph::from_source(source).unwrap();
	let after = SchemaGraph::from_source(source).unwrap();

	let diff = compare_schemas(&before, &after);
	assert!(diff.changed_fields.is_empty());
}
