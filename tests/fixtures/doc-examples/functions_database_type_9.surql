/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ age: 19, id: person:826qwse66s6igdeh977j, name: 'Galen', surname: 'Pathwarden' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ name: 'Galen', surname: 'Pathwarden' }]"

*/

LET $omit = ["id", "age"];
CREATE person SET name = "Galen", surname = "Pathwarden", age = 19;
SELECT * OMIT type::fields($omit) FROM person;
