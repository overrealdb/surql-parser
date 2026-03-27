/**[test]

[[test.results]]
value = "false"

[[test.results]]
value = "true"

[[test.results]]
value = "false"

*/

RETURN array::matches([0, 1, 2], 1);

-- [false, true, false]
