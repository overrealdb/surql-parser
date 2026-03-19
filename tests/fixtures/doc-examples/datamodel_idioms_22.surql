/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "{ location: (50, -5.4), num_people: 500 }"

*/


LET $town = {
    location: (50.0, -5.4),
    population: 500
};

SELECT 
	location,
	population AS num_people
FROM ONLY $town;
