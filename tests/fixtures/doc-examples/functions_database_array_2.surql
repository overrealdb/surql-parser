/**[test]

[[test.results]]
value = "false"

[[test.results]]
value = "true"

*/

RETURN array::all([ 1, 2, 3, NONE, 'SurrealDB', 5 ]);
-- false

RETURN ["all", "clear"].all();
-- true
