/**
[test]

[[test.results]]
value = "true"

[[test.results]]
value = "false"

[[test.results]]
value = "true"

**/

RETURN type::is_string("person:john");
RETURN type::is_record("person:john");
RETURN type::is_record(r"person:john");
