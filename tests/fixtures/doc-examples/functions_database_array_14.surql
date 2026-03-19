/**[test]

[[test.results]]
value = "false"

[[test.results]]
value = "true"

[[test.results]]
value = "true"

[[test.results]]
value = "false"

*/

RETURN array::boolean_xor([false, true, false, true], [false, false, true, true]);

-- [false, true, true, false]
