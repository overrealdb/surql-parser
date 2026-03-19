/**[test]

[[test.results]]
value = "[1, 10, 3, 4, 5]"

*/

RETURN array::fill([ 1, NONE, 3, 4, 5 ], 10, 1, 2);

-- [ 1, 10, 3, 4, 5 ]
