/**
[test]

[[test.results]]
value = "true"

[[test.results]]
value = "false"

*/
RETURN set::any({ 1, 2, 3, NONE, 'SurrealDB', 5 });
-- true

RETURN {'', 0, NONE, NULL, [], {}}.any();
-- false
