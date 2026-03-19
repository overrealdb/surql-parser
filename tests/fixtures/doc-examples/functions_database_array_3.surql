/**[test]

[[test.results]]
value = "true"

[[test.results]]
value = "true"

[[test.results]]
value = "false"

*/

RETURN ["same", "same", "same"].all("same");
-- true

[
  "What's",
  "it",
  "got",
  "in",
  "its",
  "pocketses??"
].all(|$s| $s.len() > 1);
-- true

[1, 2, "SurrealDB"].all(|$var| $var.is_string());
-- false
