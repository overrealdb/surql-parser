/**[test]

[[test.results]]
value = "true"

[[test.results]]
value = "false"

[[test.results]]
value = "false"

[[test.results]]
value = "false"

*/

RETURN array::logical_and([true, false, true, false], [true, true, false, false]);

-- [ true, false, false, false ]
