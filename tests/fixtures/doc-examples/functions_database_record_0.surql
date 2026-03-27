/**[test]

[[test.results]]
value = "false"

[[test.results]]
value = "[{ id: person:tobie }]"

[[test.results]]
value = "true"

*/

RETURN record::exists(r"person:tobie");
-- false

CREATE person:tobie;
RETURN record::exists(r"person:tobie");
-- true
