fn main() {
	let out_dir = std::env::var("OUT_DIR").unwrap();

	// Validate all .surql files at build time — fail build on invalid queries
	surql_parser::build::validate_schema_or_fail("surql/");

	// Generate typed constants for SurrealQL functions
	surql_parser::build::generate_typed_functions(
		"surql/",
		format!("{out_dir}/surql_functions.rs"),
	);
}
