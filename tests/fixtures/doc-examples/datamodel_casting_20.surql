/**[test]

[[test.results]]
value = "[user:one, [person:one, user:two]]"

*/

[
  <record<user|person>>"user:one",
  <array<record<user|person>>>["person:one", "user:two"]
];
