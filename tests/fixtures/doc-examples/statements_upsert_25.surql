/**[test]

[[test.results]]
value = ""

*/

UPSERT person:3 SET important = true WHERE ->knows->person->(knows WHERE influencer = true) TIMEOUT 5s;
