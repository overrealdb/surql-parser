/**[test]

[[test.results]]
value = "[1, 1]"

[[test.results]]
value = "[true, true, true]"

*/

RETURN array::filter([ 1, 2, 1, 3, 3, 4 ], 1);
-- [ 1, 1 ]

RETURN [true, false, false, false, true, true].filter(true);
-- [ true, true, true ]
