/**[test]

[[test.results]]
value = "true"

[[test.results]]
value = "true"

[[test.results]]
value = "false"

[[test.results]]
value = "false"

[[test.results]]
value = "false"

*/

RETURN string::is_record("person:test");           -- true
RETURN string::is_record("person:test", "person"); -- true
RETURN string::is_record("person:test", type::table("personn")); -- false
RETURN string::is_record("person:test", "other");  -- false
RETURN string::is_record("not a record id");       -- false
