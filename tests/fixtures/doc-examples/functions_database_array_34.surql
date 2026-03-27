/**[test]

[[test.results]]
value = "1"

[[test.results]]
value = "NONE"

*/

RETURN array::find_index(['a', 'b', 'c', 'b', 'a'], 'b');
-- 1

RETURN [1, 2, 3].find_index(4);
-- NONE
