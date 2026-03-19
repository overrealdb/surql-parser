/**[test]

[[test.results]]
value = "false"

[[test.results]]
value = "false"

*/

RETURN array::boolean_and([true, true], [false]);

-- [ false, false ]
