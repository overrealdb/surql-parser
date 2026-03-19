/**[test]

[[test.results]]
value = "[{ company: 'SurrealDB', id: person:tobie, interests: ['Java'], name: 'Tobie', skills: ['Rust', 'Go', 'JavaScript'] }]"

*/

-- UPSERT a document and remove a tag from an array
UPSERT person:tobie SET interests += 'Java';
