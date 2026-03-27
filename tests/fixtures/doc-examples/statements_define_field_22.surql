/**[test]

[[test.results]]
value = "NONE"

*/

-- Ensure that an email address is always stored in lowercase characters
DEFINE FIELD email ON TABLE user TYPE string
  VALUE string::lowercase($value);
