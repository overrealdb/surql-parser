/**
[test]

[[test.results]]
value = "{1, 2, 3, 4}"

[[test.results]]
value = "{1, 2, 3, 4, 5}"
*/

RETURN {1, 2}.add([2, 3, 4]);
-- {1, 2, 3, 4}

RETURN {1, 2, 3}.add({3, 4, 5});
-- {1, 2, 3, 4, 5}
