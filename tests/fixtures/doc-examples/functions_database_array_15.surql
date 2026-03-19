/**[test]

[[test.results]]
value = "true"

[[test.results]]
value = "false"

[[test.results]]
value = "true"

[[test.results]]
value = "false"

*/
RETURN array::boolean_not([ false, true, 0, 1 ]);

-- [true, false, true, false]
