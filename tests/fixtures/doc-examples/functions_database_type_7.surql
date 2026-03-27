/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ age: 19, id: person:7iucxhs7x6ausbdlhj8a, name: 'Galen', surname: 'Pathwarden' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ age: 19, name: 'Galen', surname: 'Pathwarden' }]"

*/

LET $omit = "id";
CREATE person SET name = "Galen", surname = "Pathwarden", age = 19;
SELECT * OMIT type::field($omit) FROM person;
