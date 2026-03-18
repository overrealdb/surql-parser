fn main() {
	let out_dir = std::env::var("OUT_DIR").unwrap();

	// Validate all .surql files at build time
	surql_parser::build::validate_schema("surql/");

	// Generate typed constants for SurrealQL functions
	surql_parser::build::generate_typed_functions(
		"surql/",
		format!("{out_dir}/surql_functions.rs"),
	);
}
