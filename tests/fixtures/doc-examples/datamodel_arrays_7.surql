/**[test]

[[test.results]]
value = "[3, 5]"

[[test.results]]
value = "[3, 5]"

*/

[1,3,5].filter(|$val| $val > 2);
[1,3,5][WHERE $this > 2];

-- [3,5]
