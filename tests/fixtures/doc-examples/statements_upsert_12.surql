/**[test]

[[test.results]]
value = "{ company: 'SurrealDB', id: person:tobie, name: 'Tobie', skills: ['Rust', 'Go', 'JavaScript'] }"

*/

-- UPSERT just a single record
-- Using the ONLY keyword, just an object for the record in question will be returned.
-- This, instead of an array with a single object.
UPSERT ONLY person:tobie SET 
	name = 'Tobie', 
	company = 'SurrealDB', 
	skills = ['Rust', 'Go', 'JavaScript'];
