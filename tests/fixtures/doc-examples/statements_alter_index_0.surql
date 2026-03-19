/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ id: user:317esn5r3spd4em0lfze, name: 'LordofSalty' }]"
skip-record-id-key = true

[[test.results]]
value = "NONE"

*/

DEFINE TABLE user SCHEMALESS;
DEFINE FIELD name ON TABLE user TYPE string;
CREATE user SET name = "LordofSalty";

-- Now make it schemafull to ensure that no other fields can be used
ALTER TABLE user SCHEMAFULL;
