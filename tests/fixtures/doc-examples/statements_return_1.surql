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

123;
"I am a string!";
{
	prop: "value"
};
SELECT * FROM person;
(CREATE person).id;
