/**[test]

[[test.results]]
value = "'b'"

[[test.results]]
value = "NONE"

*/

RETURN array::find(['a', 'b', 'c', 'b', 'a'], 'b');
-- b

RETURN [1, 2, 3].find(4);
-- [NONE]
