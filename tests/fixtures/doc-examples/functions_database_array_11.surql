/**[test]

[[test.results]]
value = "true"

[[test.results]]
value = "true"

[[test.results]]
value = "false"

[[test.results]]
value = "true"

*/

RETURN array::boolean_and(["true", "false", 1, 1], ["true", "true", 0, "true"]);

-- [true, true, false, true]
