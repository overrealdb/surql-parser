/**[test]

[[test.results]]
value = "`55`"

[[test.results]]
value = "cat"

[[test.results]]
value = "dog"

[[test.results]]
value = "`dog:two`"

*/

RETURN [
  type::table(55),
  type::table(cat:one),
  type::table("dog"),
  type::table("dog:two"),
];
