/**[test]

[[test.results]]
value = "[{ company: 'SurrealDB', id: person:100, name: 'Tobie', skills: ['Rust', 'Go', 'JavaScript'] }]"

*/

-- Create a new record with a numeric id
CREATE person:100 CONTENT {
	name: 'Tobie',
	company: 'SurrealDB',
	skills: ['Rust', 'Go', 'JavaScript'],
};
