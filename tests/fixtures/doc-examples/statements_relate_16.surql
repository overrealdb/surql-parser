/**[test]

[[test.results]]
value = "[]"

*/

-- Cancel this conditional filtering based on graph edge properties
-- if not finished within 5 seconds
SELECT * FROM person WHERE ->knows->person->(knows WHERE influencer = true) TIMEOUT 5s;
