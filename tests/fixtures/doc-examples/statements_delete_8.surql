/**[test]

[[test.results]]
value = "[]"

*/

DELETE person WHERE ->knows->person->(knows WHERE influencer = false) TIMEOUT 5s;
