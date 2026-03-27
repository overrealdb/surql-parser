/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE ACCESS user ON DATABASE TYPE RECORD
	SIGNUP ( CREATE user SET email = $email, pass = crypto::argon2::generate($pass) )
	SIGNIN ( SELECT * FROM user WHERE email = $email AND crypto::argon2::compare(pass, $pass) )
	WITH REFRESH
	DURATION FOR GRANT 15d, FOR TOKEN 1m, FOR SESSION 12h
;
