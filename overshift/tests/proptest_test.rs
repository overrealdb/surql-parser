use proptest::prelude::*;

proptest! {
	#[test]
	fn manifest_toml_parsing_never_panics(s in "\\PC{0,500}") {
		let _ = toml::from_str::<overshift::Manifest>(&s);
	}

	#[test]
	fn manifest_with_valid_meta_parses(
		ns in "[a-z]{1,10}",
		db in "[a-z]{1,10}",
		system_db in "[a-z]{1,10}",
	) {
		let toml_str = format!(
			"[meta]\nns = \"{ns}\"\ndb = \"{db}\"\nsystem_db = \"{system_db}\"\n"
		);
		let manifest: overshift::Manifest = toml::from_str(&toml_str).unwrap();
		prop_assert_eq!(&manifest.meta.ns, &ns);
		prop_assert_eq!(&manifest.meta.db, &db);
		prop_assert_eq!(&manifest.meta.system_db, &system_db);
	}

	#[test]
	fn compute_checksum_never_panics(s in "\\PC{0,1000}") {
		let hash = overshift::compute_checksum(&s);
		prop_assert_eq!(hash.len(), 64);
	}
}
