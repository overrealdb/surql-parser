/**[test]

[[test.results]]
value = "true"

[[test.results]]
value = "false"

[[test.results]]
value = "true"

*/

RETURN ["same", "same?", "Dude, same!"].any("same");
-- true

[
  "What's",
  "it",
  "got",
  "in",
  "its",
  "pocketses??"
].any(|$s| $s.len() > 15);
-- false

[1, 2, "SurrealDB"].any(|$var| $var.is_string());
-- true
