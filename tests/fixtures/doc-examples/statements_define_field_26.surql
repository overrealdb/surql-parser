/**[test]

[[test.results]]
value = "NONE"

*/

-- Create a field if it does not already exist
DEFINE FIELD IF NOT EXISTS email ON TABLE user TYPE string;
