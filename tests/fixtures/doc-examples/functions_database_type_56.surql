/**[test]

[[test.results]]
value = "false"

[[test.results]]
value = "false"

*/

-- Traditional syntax
type::is_record(r"person:aeon", "cat");

-- Method chaining syntax
r"person:aeon".is_record("cat");
