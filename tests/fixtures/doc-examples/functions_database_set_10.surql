/**
[test]

[[test.results]]
value = "'b'"

[[test.results]]
value = "NONE"

*/
RETURN set::find({'a', 'b', 'c'}, 'b');
-- 'b'

RETURN {1, 2, 3}.find(4);
-- NONE
