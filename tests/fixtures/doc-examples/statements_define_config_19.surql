/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ email: 'user@example.com', id: foo:1, val: 42 }]"

[[test.results]]
value = "[{ email: 'other@example.com', id: foo:2, val: 43 }]"

*/

-- Define a user with access permissions
DEFINE USER my_user ON DATABASE PASSWORD 'my_password';
DEFINE ACCESS user ON DATABASE TYPE RECORD
  SIGNUP ( CREATE user SET email = $email, pass = crypto::argon2::generate($pass) )
  SIGNIN ( SELECT * FROM user WHERE email = $email AND crypto::argon2::compare(pass, $pass) )
  DURATION FOR SESSION 60s, FOR TOKEN 1d;

-- Define a table with permissions
DEFINE TABLE foo SCHEMAFUL PERMISSIONS FOR select WHERE $auth.email = email;
DEFINE FIELD email ON foo TYPE string;
DEFINE FIELD val ON foo TYPE int;

-- Insert data
CREATE foo:1 SET val = 42, email = "user@example.com";
CREATE foo:2 SET val = 43, email = "other@example.com";
