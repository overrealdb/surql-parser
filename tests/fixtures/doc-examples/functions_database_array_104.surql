/**[test]

[[test.results]]
value = "['something', 4, 3, 3, 2, 1, 1, 0, NULL]"

*/

RETURN array::sort::desc([ 1, 2, 1, null, "something", 3, 3, 4, 0 ]);

-- [ "something", 4, 3, 3, 2, 1, 1, 9, null ]
