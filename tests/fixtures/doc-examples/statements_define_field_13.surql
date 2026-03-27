/**[test]

[[test.results]]
value = "NONE"

*/

-- A user is not locked by default.
DEFINE FIELD locked ON TABLE user TYPE bool
-- Set a default value if empty
  DEFAULT false;
