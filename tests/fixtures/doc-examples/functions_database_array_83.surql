/**[test]

[[test.results]]
value = "[1, 2, 3, 5]"

*/

RETURN array::remove([1, 2, 3, 4, 5], -2);

-- [ 1, 2, 3, 5 ]
