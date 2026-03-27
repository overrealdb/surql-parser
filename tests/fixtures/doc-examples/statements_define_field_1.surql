/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

*/

-- Define nested object property types
DEFINE FIELD emails.address ON TABLE user TYPE string;
DEFINE FIELD emails.primary ON TABLE user TYPE bool;

-- Define individual fields on an array
DEFINE FIELD metadata[0] ON person TYPE datetime;
DEFINE FIELD metadata[1] ON person TYPE int;
