/**[test]

[[test.results]]
value = "false"

[[test.results]]
value = "true"

[[test.results]]
value = "true"

[[test.results]]
value = "true"

*/

RETURN array::boolean_or([false, true, false, true], [false, false, true, true]);

-- [false, true, true, true]
