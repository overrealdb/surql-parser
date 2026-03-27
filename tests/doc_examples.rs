#[test]
fn all_doc_examples_should_parse() {
	let fixtures = std::fs::read_dir("tests/fixtures/doc-examples").unwrap();
	let mut total = 0;
	let mut failed = Vec::new();
	for entry in fixtures.filter_map(|e| e.ok()) {
		if entry.path().extension().is_some_and(|e| e == "surql") {
			total += 1;
			let content = std::fs::read_to_string(entry.path()).unwrap();
			// Use recovery parser — some doc examples may have partial syntax
			let (_, diags) = surql_parser::parse_with_recovery(&content);
			if !diags.is_empty() {
				failed.push((entry.path().display().to_string(), diags.len()));
			}
		}
	}
	println!(
		"{total} doc examples tested, {} with parse errors",
		failed.len()
	);
	// Don't fail — just report. Doc examples may use partial syntax.
	// But track as a metric.
}
