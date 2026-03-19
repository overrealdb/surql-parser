/**[test]

[[test.results]]
value = "NONE"

*/

-- Set a field to have either the uuid or int type
DEFINE FIELD user_id ON TABLE user TYPE uuid|int;
