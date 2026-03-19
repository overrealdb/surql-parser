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
value = "[{ first_name: 'bob', full_name: 'bob BOBSON', id: person:l07j1ly4oher21g80fr4, last_name: 'bobson' }]"
skip-record-id-key = true

*/

DEFINE TABLE person SCHEMAFULL;

DEFINE FIELD first_name ON TABLE person TYPE string VALUE string::lowercase($value);
DEFINE FIELD last_name  ON TABLE person TYPE string VALUE string::lowercase($value);
DEFINE FIELD full_name  ON TABLE person             VALUE first_name + ' ' + last_name;

// Creates a `person` with `full_name` of "bob BOBSON", not "bob bobson"
CREATE person SET first_name = "Bob", last_name = "BOBSON";
