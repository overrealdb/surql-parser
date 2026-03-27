/**[test]

[[test.results]]
value = "{ a: 1, b: true }"

*/

RETURN object::from_entries([
  [ "a", 1 ],
  [ "b", true ],
]);
