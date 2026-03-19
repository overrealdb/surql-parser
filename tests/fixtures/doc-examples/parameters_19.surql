/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE TABLE user SCHEMAFULL
  PERMISSIONS FOR select, update, delete, create
  WHERE $access = "users"
  AND email = $token.email;
