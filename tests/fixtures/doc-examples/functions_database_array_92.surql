/**[test]

[[test.results]]
value = "[3, 4, 5]"

*/

RETURN array::slice([ 1, 2, 3, 4, 5 ], 2);

-- [ 3, 4, 5 ]
