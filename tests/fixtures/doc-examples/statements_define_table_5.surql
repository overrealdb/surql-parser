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
value = "[{ first_name: 'Bob', id: person:5o2zt3el25z7kbuzsf47, last_name: 'Bobson', name: 'Bob Bobson' }]"
skip-record-id-key = true

*/

DEFINE TABLE person SCHEMAFULL;

DEFINE FIELD first_name ON TABLE person TYPE string ASSERT string::len($value) < 20;
DEFINE FIELD last_name  ON TABLE person TYPE string ASSERT string::len($value) < 20;
DEFINE FIELD name       ON TABLE person             VALUE first_name + ' ' + last_name;

// Creates a `person` with the name "Bob Bobson"
CREATE person SET first_name = "Bob", last_name = "Bobson";
