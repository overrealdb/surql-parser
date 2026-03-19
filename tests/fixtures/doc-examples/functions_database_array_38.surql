/**[test]

[[test.results]]
value = "[1, 2, 3, 4, 'SurrealDB', 5, 6, [7, 8]]"

*/

RETURN array::flatten([ [1, 2], [3, 4], 'SurrealDB', [5, 6, [7, 8]] ]);
