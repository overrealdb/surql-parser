/**[test]

[[test.results]]
value = "[{ age: 19, id: person:v5tk0ctrk8vjgpsjsr2n, name: 'Galen', surname: 'Pathwarden' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ age: 19, surname: 'Pathwarden' }]"

*/

CREATE person SET name = "Galen", surname = "Pathwarden", age = 19;

SELECT * OMIT type::fields(["name", "id"]) FROM person;
