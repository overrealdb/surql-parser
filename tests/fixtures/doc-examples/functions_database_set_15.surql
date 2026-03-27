/**
[test]

[[test.results]]
value = "'123'"
*/

{1, 2, 3}.fold('', |$acc, $val| $acc + <string>$val);
-- '123'
