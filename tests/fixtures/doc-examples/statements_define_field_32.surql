/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ first_name: 'bob', id: person:bpkup0u5zv84xqxne2j5, last_name: 'bobson', name: 'bob bobson' }]"
skip-record-id-key = true

*/

DEFINE TABLE person SCHEMAFULL;

DEFINE FIELD first_name ON TABLE person TYPE string VALUE string::lowercase($value);
DEFINE FIELD last_name  ON TABLE person TYPE string VALUE string::lowercase($value);
DEFINE FIELD name       ON TABLE person             VALUE first_name + ' ' + last_name;

// Creates a `person` with the name "bob bobson"
CREATE person SET first_name = "BOB", last_name = "BOBSON";
