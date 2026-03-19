/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ age: 90, created: d'2024-01-01T00:00:00Z', id: person:gladys }]"

[[test.results]]
value = "[{ age: 70, created: d'2024-01-01T00:00:00Z', id: person:gladys }]"

*/

DEFINE FIELD created ON person TYPE datetime DEFAULT d'2024-01-01T00:00:00Z' READONLY;
CREATE person:gladys SET age = 90;
-- Does not try to modify `created` field, no error
UPDATE person:gladys CONTENT { age: 70 };
