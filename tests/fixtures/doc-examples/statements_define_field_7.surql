/**[test]

[[test.results]]
value = "NONE"

*/

-- A user may enter a biography, but it is not required.
-- By using the option type you also allow for NONE values.
DEFINE FIELD biography ON TABLE user TYPE option<string>;
