/**[test]

[[test.results]]
value = "123"

[[test.results]]
value = "'I am a string!'"

[[test.results]]
value = "{ prop: 'value' }"

[[test.results]]
value = "[]"

[[test.results]]
value = "[person:mbn3r0epzxiz5hoqr2ls]"
skip-record-id-key = true

*/

-- Return a simple value
RETURN 123;
RETURN "I am a string!";
RETURN {
	prop: "value"
};

-- Return the result of a query
RETURN SELECT * FROM person;
RETURN (CREATE person).id;
