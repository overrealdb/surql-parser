/**[test]

[[test.results]]
value = "true"

[[test.results]]
value = "false"

*/

[1, 2, 3].some(|$num| $num > 2);
-- true

[1999, 2001, 2002].includes(2000);
-- false
