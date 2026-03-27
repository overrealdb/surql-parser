/**[test]

[[test.results]]
value = "NONE"

*/

-- In this example, we throw a custom error when a user provides invalid signin details
DEFINE ACCESS user ON DATABASE TYPE RECORD
	SIGNIN {
		LET $user = (SELECT * FROM user WHERE username = $username AND crypto::argon2::compare(password, $password));
		IF !$user {
			THROW "You either provided invalid credentials, or a user with the username " + <string> $username + " might not exist.";
		};

		RETURN $user;
	}
	DURATION FOR SESSION 1w
;
