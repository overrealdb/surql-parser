/**[test]

[[test.results]]
value = "[3, 5, 4, 1, 2]"

*/

RETURN array::shuffle([ 1, 2, 3, 4, 5 ]);

-- [ 2, 1, 4, 3, 5 ]
