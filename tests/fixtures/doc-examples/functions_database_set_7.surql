/**
[test]

[[test.results]]
value = "3"

[[test.results]]
value = "NONE"

*/

RETURN {1, 2, 3}.at(-1);
-- 3

RETURN {1, 2, 3}.at(-4);
-- NONE
