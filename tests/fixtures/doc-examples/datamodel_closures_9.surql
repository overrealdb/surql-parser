/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[[1]]"

*/

LET $okay_nums = [1,2,3];

[1,5,6,7,0].filter(|$n| $n IN $okay_nums);
