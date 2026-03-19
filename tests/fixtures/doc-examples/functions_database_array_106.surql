/**[test]

[[test.results]]
value = "[5, 2, 3, 4, 1]"

*/

RETURN array::swap([ 1, 2, 3, 4, 5 ], 0, -1);

-- [ 5, 2, 3, 4, 1 ]
