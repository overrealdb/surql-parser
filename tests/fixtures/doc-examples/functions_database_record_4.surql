/**[test]

[[test.results]]
value = "[{ id: likes:first_like, in: person:one, out: person:two }]"

[[test.results]]
value = "true"

[[test.results]]
value = "true"

*/

RELATE person:one->likes:first_like->person:two;

-- Both return true
record::is_edge(likes:first_like);
record::is_edge("likes:first_like");
