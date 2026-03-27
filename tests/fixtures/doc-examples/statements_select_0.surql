/**[test]

[[test.results]]
value = "[{ address: '1 Bagshot Row', email: 'tobie@surrealdb.com', id: person:tobie, name: { first: 'Tobie' } }]"

[[test.results]]
value = "[{ address: '1 Bagshot Row', email: 'tobie@surrealdb.com', id: person:tobie, name: { first: 'Tobie' } }]"

[[test.results]]
value = "[{ address: '1 Bagshot Row', email: 'tobie@surrealdb.com', name: { first: 'Tobie' } }]"

[[test.results]]
value = "[{ address: '1 Bagshot Row', email: 'tobie@surrealdb.com', id: person:tobie, name: { first: 'Tobie' } }]"

[[test.results]]
value = "[{ address: '1 Bagshot Row', email: 'tobie@surrealdb.com', name: { first: 'Tobie' } }]"

[[test.results]]
value = "{ address: '1 Bagshot Row', email: 'tobie@surrealdb.com', id: person:tobie, name: { first: 'Tobie' } }"

*/

CREATE person:tobie SET
	name.first = "Tobie",
	address = "1 Bagshot Row",
	email = "tobie@surrealdb.com";

-- Select all fields from a table
SELECT * FROM person;

-- Select specific fields from a table
SELECT name, address, email FROM person;

-- Select all fields from a specific record
SELECT * FROM person:tobie;

-- Select specific fields from a specific record
SELECT name, address, email FROM person:tobie;

-- Select just a single record
-- Using the ONLY keyword, just an object
-- for the record in question will be returned.
-- This, instead of an array with a single object.
SELECT * FROM ONLY person:tobie;
