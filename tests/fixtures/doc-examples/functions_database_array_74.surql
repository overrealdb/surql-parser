/**[test]

[[test.results]]
value = "[5, 1, 2, 3, 4]"

*/

RETURN array::prepend([1, 2, 3, 4], 5);

-- [ 5, 1, 2, 3, 4 ]
