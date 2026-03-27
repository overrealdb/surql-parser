/**[test]

[[test.results]]
value = "NONE"

*/

-- Give the user table an email field. Store it in a string
DEFINE FIELD email ON TABLE user TYPE string
  -- Check if the value is a properly formatted email address
  ASSERT string::is_email($value);
