/**
[test]

[[test.results]]
value = "true"

[[test.results]]
value = "true"

[[test.results]]
value = "false"

*/

RETURN {'same', 'different'}.any('same');
-- true

RETURN {'ant', 'bear', 'cat'}.any(|$s| $s.len() > 3);
-- true

RETURN {1, 2, 3}.any(|$num| $num > 10);
-- false
