/**[test]

[[test.results]]
value = "[{ id: temperature:['London', d'2025-02-19T00:00:00Z'], val: 5.5f }]"

[[test.results]]
value = "[{ id: temperature:['London', d'2025-02-20T00:00:00Z'], val: 5.7f }]"

[[test.results]]
value = "[{ id: temperature:['London', d'2025-02-19T00:00:00Z'], val: 5.5f }, { id: temperature:['London', d'2025-02-20T00:00:00Z'], val: 5.7f }]"

*/

CREATE temperature:['London', d'2025-02-19T00:00:00.000Z'] SET val = 5.5;
CREATE temperature:['London', d'2025-02-20T00:00:00.000Z'] SET val = 5.7;

-- Return all records as long as index 0 = 'London'
SELECT * FROM temperature:['London', NONE]..=['London', ..];
