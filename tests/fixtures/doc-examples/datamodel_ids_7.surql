/**[test]

[[test.results]]
value = "[{ conditions: 'cloudy', id: weather:['London', d'2025-02-13T05:00:00Z'], temperature: 5.7f }]"

*/

CREATE weather:['London', d'2025-02-13T05:00:00Z'] SET
    temperature = 5.7,
    conditions = "cloudy";
