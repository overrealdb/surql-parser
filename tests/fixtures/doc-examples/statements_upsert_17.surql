/**[test]

[[test.results]]
value = "[{ company: 'SurrealDB', id: person:6vfpq64z5r6bpe8cb91q, name: 'Tobie', skills: ['Rust', 'Go', 'JavaScript'] }]"

[[test.results]]
value = "[{ company: 'SurrealDB', id: person:tobie, name: 'Tobie', skills: ['Rust', 'Go', 'JavaScript'] }]"

*/

-- UPSERT all records with the same content
UPSERT person CONTENT {
	name: 'Tobie',
	company: 'SurrealDB',
	skills: ['Rust', 'Go', 'JavaScript'],
};

-- UPSERT a specific record with some content
UPSERT person:tobie CONTENT {
	name: 'Tobie',
	company: 'SurrealDB',
	skills: ['Rust', 'Go', 'JavaScript'],
};
