/**[test]

[[test.results]]
value = "true"

[[test.results]]
value = "false"

*/

RETURN array::any([ 1, 2, 3, NONE, 'SurrealDB', 5 ]);
-- true

["", 0, NONE, NULL, [], {}].any();
-- false
