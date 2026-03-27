/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ conditions: 'cloudy', id: weather:['Seoul', d'2025-10-03T01:11:38.204589Z'], temperature: -2.3f }]"
skip-datetime = true

[[test.results]]
value = "[{ conditions: 'cloudy', id: weather:['London', d'2025-10-03T01:11:38.204961Z'], temperature: 5.3f }]"
skip-datetime = true

*/

LET $now = time::now();

CREATE weather:['Seoul', $now] SET
    temperature = -2.3,
    conditions = "cloudy";

CREATE weather:['London', time::now()] SET
    temperature = 5.3,
    conditions = "cloudy";
