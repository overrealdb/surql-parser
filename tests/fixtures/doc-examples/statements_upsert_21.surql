/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ age: 90, created: d'2024-01-01T00:00:00Z', id: person:gladys }]"

[[test.results]]
error = "'Found changed value for field `created`, with record `person:gladys`, but field is readonly'"

[[test.results]]
value = "[{ age: 70, created: d'2024-01-01T00:00:00Z', id: person:gladys }]"

*/

DEFINE FIELD created ON person TYPE datetime DEFAULT d'2024-01-01T00:00:00Z' READONLY;
UPSERT person:gladys SET age = 90;
-- Attempts to change `created` field, error
UPSERT person:gladys REPLACE { age: 70 };
-- `created` equals current value, query works
UPSERT person:gladys REPLACE { age: 70, created: d'2024-01-01T00:00:00Z' };
