/**[test]

[[test.results]]
value = [3, 4]""

*/

RETURN array::intersect([1, 2, 3, 4], [3, 4, 5, 6]);

-- [ 3, 4 ]
