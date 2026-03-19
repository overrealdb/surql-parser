/**
[test]

[[test.results]]
value = "{1, 4}"

[[test.results]]
value = "{1}"

*/

RETURN {1, 2, 3, 4}.remove([2, 3]);
-- {1, 4}

RETURN {1, 2, 3, 4}.remove({2, 3, 4});
-- {1,}
