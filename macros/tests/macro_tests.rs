#[test]
fn compile_tests() {
	let t = trybuild::TestCases::new();
	t.pass("tests/ui/pass_*.rs");
	t.compile_fail("tests/ui/fail_*.rs");
}

#[test]
fn schema_validation_tests() {
	let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| {
		let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
		// macros/ is one level below workspace root; workspace target is at root
		std::path::Path::new(&manifest)
			.parent()
			.unwrap()
			.join("target")
			.to_str()
			.unwrap()
			.to_string()
	});
	let trybuild_dir = std::path::PathBuf::from(&target_dir).join("tests/trybuild/surql-macros");

	let surql_dir = trybuild_dir.join("surql_fixtures");
	std::fs::create_dir_all(&surql_dir).unwrap();
	std::fs::write(
		surql_dir.join("functions.surql"),
		"DEFINE FUNCTION OVERWRITE fn::add($a: int, $b: int) -> int { RETURN $a + $b; };\n\
		 DEFINE FUNCTION OVERWRITE fn::greet($name: string) -> string { RETURN 'Hello, ' + $name; };\n\
		 DEFINE FUNCTION OVERWRITE fn::get_user($id: record<user>) -> object { RETURN SELECT * FROM $id; };\n\
		 DEFINE FUNCTION OVERWRITE fn::toggle($flag: bool) -> bool { RETURN !$flag; };\n\
		 DEFINE FUNCTION OVERWRITE fn::scale($value: float, $factor: number) -> float { RETURN $value * $factor; };\n\
		 DEFINE FUNCTION OVERWRITE fn::search($tags: array<string>) -> array { RETURN SELECT * FROM post WHERE tags CONTAINSANY $tags; };\n\
		 DEFINE FUNCTION OVERWRITE fn::optional_greet($name: option<string>) -> string { RETURN 'Hello, ' + $name ?? 'World'; };\n",
	)
	.unwrap();
	std::fs::write(
		surql_dir.join("schema.surql"),
		"DEFINE TABLE OVERWRITE user SCHEMAFULL;\n\
		 DEFINE FIELD OVERWRITE name ON user TYPE string;\n\
		 DEFINE FIELD OVERWRITE age ON user TYPE int;\n\
		 DEFINE FIELD OVERWRITE email ON user TYPE string;\n\
		 DEFINE FIELD OVERWRITE active ON user TYPE bool;\n\
		 DEFINE FIELD OVERWRITE score ON user TYPE float;\n\
		 DEFINE FIELD OVERWRITE tags ON user TYPE array<string>;\n\
		 DEFINE FIELD OVERWRITE bio ON user TYPE none | string;\n\
		 DEFINE FIELD OVERWRITE role ON user TYPE string;\n\
		 DEFINE FIELD OVERWRITE created_at ON user TYPE datetime;\n\
		 DEFINE TABLE OVERWRITE post SCHEMAFULL;\n\
		 DEFINE FIELD OVERWRITE title ON post TYPE string;\n\
		 DEFINE FIELD OVERWRITE author ON post TYPE record<user>;\n\
		 DEFINE FIELD OVERWRITE views ON post TYPE int;\n",
	)
	.unwrap();

	let t = trybuild::TestCases::new();
	t.pass("tests/schema_ui/pass_function_schema_match.rs");
	t.pass("tests/schema_ui/pass_function_type_match.rs");
	t.pass("tests/schema_ui/pass_query_schema_match.rs");
	t.compile_fail("tests/schema_ui/fail_function_arity_mismatch.rs");
	t.compile_fail("tests/schema_ui/fail_function_type_mismatch.rs");
	t.compile_fail("tests/schema_ui/fail_query_type_mismatch.rs");
}
