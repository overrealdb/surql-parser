/**[test]

[[test.results]]
value = "false"

[[test.results]]
value = "false"

[[test.results]]
value = "true"

*/

RETURN array::all(["", 1, 2, 3]); // false because of ""
RETURN array::all([{}, 1, 2, 3]); // false because of {}
RETURN array::all(["SurrealDB", { is_nice_database: true }, 1, 2, 3]);  // true
