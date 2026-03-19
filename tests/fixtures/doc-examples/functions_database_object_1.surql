/**[test]

[[test.results]]
value = "{ gold: 100, last_updated: d'2025-10-07T02:36:30.164629Z', name: 'Mat Cauthon' }"
skip-datetime = true

*/

{ name: "Mat Cauthon", last_updated: d'2013-01-08'}.extend( 
{ gold: 100, last_updated: time::now() });
