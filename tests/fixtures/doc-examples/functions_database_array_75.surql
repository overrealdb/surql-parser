/**[test]

[[test.results]]
value = "[1, 2, 3, 4, 5]"

*/

RETURN array::push([1, 2, 3, 4], 5);

-- [ 1, 2, 3, 4, 5 ]
