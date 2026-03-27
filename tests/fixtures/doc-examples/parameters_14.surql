/**[test]

[[test.results]]
value = "[{ historical_data: [], id: city:london, population: 8900000, year: 2019 }]"

[[test.results]]
value = "[{ historical_data: [{ population: 8900000, year: 2019 }], id: city:london, population: 9600000, year: 2023 }]"

*/

CREATE city:london SET
    population = 8900000,
    year = 2019,
    historical_data = [];

INSERT INTO city [
    { id: "london", population: 9600000, year: 2023 }
]
ON DUPLICATE KEY UPDATE
-- Stick old data into historical_data
historical_data += {
    year: year,
    population: population
},
-- Then update current record with the new input using $input
population = $input.population,
year = $input.year;
