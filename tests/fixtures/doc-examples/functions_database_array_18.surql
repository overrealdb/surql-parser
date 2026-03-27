/**[test]

[[test.results]]
value = "[1, 2, 3, 4, 3, 4, 5, 6]"

*/

RETURN array::concat([1, 2, 3, 4], [3, 4, 5, 6]);
-- [1, 2, 3, 4, 3, 4, 5, 6]

RETURN [1,2].concat([3,4], [4,3])
-- [1, 2, 3, 4, 4, 3]
