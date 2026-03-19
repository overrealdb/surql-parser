/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ address: '1 Bagshot Row', email: 'tobie@surrealdb.com', id: person:tobie, name: 'Tobie' }]"

[[test.results]]
value = "[{ detail: { plan: { index: 'fast_email', operator: '=', value: 'tobie@surrealdb.com' }, table: 'person' }, operation: 'Iterate Index' }, { detail: { type: 'Memory' }, operation: 'Collector' }]"

[[test.results]]
value = "[{ detail: { plan: { index: 'fast_email', operator: '=', value: 'tobie@surrealdb.com' }, table: 'person' }, operation: 'Iterate Index' }, { detail: { type: 'Memory' }, operation: 'Collector' }, { detail: { type: 'KeysAndValues' }, operation: 'RecordStrategy' }, { detail: { count: 1 }, operation: 'Fetch' }]"

*/

DEFINE INDEX fast_email ON TABLE person FIELDS email;

CREATE person:tobie SET
	name = "Tobie",
	address = "1 Bagshot Row",
	email = "tobie@surrealdb.com";

SELECT * FROM person WHERE email='tobie@surrealdb.com' EXPLAIN;
SELECT * FROM person WHERE email='tobie@surrealdb.com' EXPLAIN FULL;
