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

RETURN array::logical_xor([true, false, true, false], [true, true, false, false]);

-- [ false, true, true, false ]
