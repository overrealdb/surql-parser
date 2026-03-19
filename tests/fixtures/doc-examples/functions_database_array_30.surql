/**[test]

[[test.results]]
value = "[1, 3]"

[[test.results]]
value = "[0, 1, 3, 4]"

*/

RETURN array::filter_index(['a', 'b', 'c', 'b', 'a'], 'b');
-- [ 1, 3 ]

RETURN [0, 0, 1, 0, 0, 5, 1].filter_index(0);
-- [ 0, 1, 3, 4 ]
