/**[test]

[[test.results]]
value = "'true'"

*/

[NONE, NONE, "good data", "Also good", "important", NULL]
    .filter(|$v| $v.is_string())
    .all(|$s| $s.len() > 5)
    .chain(|$v| <string>$v);
