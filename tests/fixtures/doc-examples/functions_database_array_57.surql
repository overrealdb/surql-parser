/**[test]

[[test.results]]
value = "true"

[[test.results]]
value = "true"

[[test.results]]
value = "true"

[[test.results]]
value = "false"

*/

RETURN array::logical_or([true, false, true, false], [true, true, false, false]);

-- [ true, true, true, false ]
