/**[test]

[[test.results]]
value = "{ id: person:tobie }"

[[test.results]]
value = "{ id: person:tobie }"

*/

CREATE ONLY person:tobie;
DELETE ONLY person:tobie RETURN BEFORE;
-- { id: person:tobie }
