/**
[test]
[[test.results]]
value = "'1234'"
*/

{1, 2, 3, 4}.reduce(|$one, $two| <string>$one + <string>$two);
-- '1234'
