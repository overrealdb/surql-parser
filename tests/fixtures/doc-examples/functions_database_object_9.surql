/**[test]

[[test.results]]
value = "{ last_updated: d'2013-01-08T00:00:00Z', name: 'Mat Cauthon' }"

[[test.results]]
value = "{ name: 'Mat Cauthon' }"

*/

{ name: "Mat Cauthon", last_updated: d'2013-01-08', gold: 100 }.remove("gold");
{ name: "Mat Cauthon", last_updated: d'2013-01-08', gold: 100 }.remove(["gold", "last_updated"]);
