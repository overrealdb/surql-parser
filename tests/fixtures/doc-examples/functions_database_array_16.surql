/**[test]

[[test.results]]
value = "[[[1, 2], [1, 3], [2, 2], [2, 3]]]"

*/

RETURN array::combine([1, 2], [2, 3]);

-- [ [1, 2], [1, 3], [2, 2], [2, 3] ]
