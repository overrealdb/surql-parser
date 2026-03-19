/**[test]

[[test.results]]
value = "NONE"

*/

-- Set permissions for the email field
DEFINE FIELD email ON TABLE user
  PERMISSIONS
    FOR select WHERE published=true OR user=$auth.id
    FOR update WHERE user=$auth.id OR $auth.role="admin";
