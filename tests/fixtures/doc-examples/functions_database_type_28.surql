/**[test]

[[test.results]]
value = "[person, cat]"

*/

RETURN [
  type::table("person"),
  type::table(cat:one)
];

-- [person, cat]
