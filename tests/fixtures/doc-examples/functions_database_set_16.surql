/**
[test]

[[test.results]]
value = "'1 + 2 + 3'"
*/

RETURN set::join({3, 1, 2}, ' + ');
-- '1 + 2 + 3'
