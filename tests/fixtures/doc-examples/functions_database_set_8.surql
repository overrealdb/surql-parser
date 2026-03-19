/**
[test]

[[test.results]]
value = "{0, 1, 2, 3}"

*/
{1, 2, 3, NONE, 0, '', [], {}}.filter(|$v| $v.is_int());
-- {0, 1, 2, 3}
