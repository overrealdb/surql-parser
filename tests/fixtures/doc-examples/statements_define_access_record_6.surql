/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE ACCESS user ON DATABASE TYPE RECORD
    SIGNUP ( CREATE user SET email = $email, pass = crypto::argon2::generate($pass), enabled = true )
    SIGNIN ( SELECT * FROM user WHERE email = $email AND crypto::argon2::compare(pass, $pass) )
    AUTHENTICATE {
        IF !$auth.enabled {
            THROW "This user is not enabled";
        };

        RETURN $auth;
    }
;
