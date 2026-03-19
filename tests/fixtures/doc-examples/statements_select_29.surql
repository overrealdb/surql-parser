/**[test]

[[test.results]]
value = "[{ address: '1 Bagshot Row', email: 'tobie@surrealdb.com', id: person:tobie, name: 'Tobie' }]"

[[test.results]]
value = "[{ detail: { direction: 'forward', table: 'person' }, operation: 'Iterate Table' }, { detail: { type: 'Memory' }, operation: 'Collector' }]"

[[test.results]]
value = "[{ detail: { direction: 'forward', table: 'person' }, operation: 'Iterate Table' }, { detail: { type: 'Memory' }, operation: 'Collector' }, { detail: { type: 'KeysAndValues' }, operation: 'RecordStrategy' }, { detail: { count: 1 }, operation: 'Fetch' }]"

*/

CREATE person:tobie SET
	name = "Tobie",
	address = "1 Bagshot Row",
	email = "tobie@surrealdb.com";

SELECT * FROM person WHERE email='tobie@surrealdb.com' EXPLAIN;
SELECT * FROM person WHERE email='tobie@surrealdb.com' EXPLAIN FULL;
