/**[test]

[[test.results]]
value = "[1, 2, 6, 3, 4, 5]"

*/

RETURN array::union([1, 2, 1, 6], [1, 3, 4, 5, 6]);

-- [ 1, 2, 6, 3, 4, 5 ]
