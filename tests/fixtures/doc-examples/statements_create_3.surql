/**[test]

[[test.results]]
value = "[{ company: 'SurrealDB', id: person:tobie, name: 'Tobie', skills: ['Rust', 'Go', 'JavaScript'] }]"

*/

-- Create a new record with a text id
CREATE person:tobie SET
    name = 'Tobie',
    company = 'SurrealDB',
    skills = ['Rust', 'Go', 'JavaScript'];
