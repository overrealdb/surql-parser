/**[test]

[[test.results]]
value = "NONE"

*/

-- Create a RECORD access method for accounts
-- On successful authentication, a token expiring after 15 minutes will be returned
-- This token can be used to establish a session that will expire after 6 hours
-- The token will be automatically used to authenticate the session
DEFINE ACCESS account ON DATABASE TYPE RECORD
	SIGNUP ( CREATE user SET email = $email, pass = crypto::argon2::generate($pass) )
	SIGNIN ( SELECT * FROM user WHERE email = $email AND crypto::argon2::compare(pass, $pass) )
	DURATION FOR TOKEN 15m, FOR SESSION 12h
;
