/**[test]

[[test.results]]
value = "[1, 2, 5, 6]"

*/

RETURN array::difference([1, 2, 3, 4], [3, 4, 5, 6]);

-- [ 1, 2, 5, 6 ]
