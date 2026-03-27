/**[test]

[[test.results]]
value = "[1, 2, 10, 4, 5]"

*/

RETURN array::fill([ 1, 2, NONE, 4, 5 ], 10, -3, -2);

-- [ 1, 2, 10, 4, 5 ]
