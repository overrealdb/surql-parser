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

DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD name ON TABLE user TYPE string;
DEFINE FIELD metadata ON TABLE user FLEXIBLE TYPE object;
DEFINE FIELD metadata.user_id ON TABLE user TYPE int;
DEFINE FIELD name ON user TYPE string;
DEFINE FIELD metadata ON TABLE user TYPE object FLEXIBLE;
DEFINE FIELD metadata.user_id ON TABLE user TYPE int;
