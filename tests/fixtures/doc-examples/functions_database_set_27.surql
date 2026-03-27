/**
[test]

[[test.results]]
value = "{2, 3}"

[[test.results]]
value = "{3, 4}"

*/

RETURN set::slice({4, 1, 3, 2}, 1, 3);
-- {2, 3}

RETURN {1, 2, 3, 4, 5}.slice(-3..);
-- {3, 4, 5}
