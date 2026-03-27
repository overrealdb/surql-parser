/**
[test]

[[test.results]]
value = "true"

[[test.results]]
value = "true"

[[test.results]]
value = "false"
*/

RETURN {'same',}.all('same');
-- true

RETURN {"What's", 'it', 'got', 'in', 'its', 'pocketses??'}.all(|$s| $s.len() > 1);
-- true

RETURN {1, 2, 'SurrealDB'}.all(|$var| $var.is_string());
-- false
