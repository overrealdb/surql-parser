/**[test]

[[test.results]]
value = "[{ at_year: 2024, id: city:Calgary, population: 1665000 }]"

*/

INSERT INTO city (id, population, at_year) VALUES ("Calgary", 1665000, 2024)
ON DUPLICATE KEY UPDATE
	population = $input.population,
	at_year = $input.at_year;
