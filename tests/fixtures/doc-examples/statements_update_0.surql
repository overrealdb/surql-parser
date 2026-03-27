/**[test]

[[test.results]]
value = "[{ company: 'Surrealist', id: person:1iberbyuqfpoi6girgv9, name: 'John', skills: ['JavaScript', 'Go', 'SurrealQL'] }]"
skip-record-id-key = true

[[test.results]]
value = "[{ company: 'SurrealDB', id: person:tobie, name: 'Tobie', skills: ['JavaScript', 'Go', 'SurrealQL'] }]"

*/

-- Create a Schemaless person table with a random id
CREATE person CONTENT {
    name: 'John',
    company: 'Surrealist',
    skills: ['JavaScript', 'Go' , 'SurrealQL']
};

-- Create another person with a specific id
CREATE person:tobie CONTENT {
    name: 'Tobie',
    company: 'SurrealDB',
    skills: ['JavaScript', 'Go' , 'SurrealQL']
};
