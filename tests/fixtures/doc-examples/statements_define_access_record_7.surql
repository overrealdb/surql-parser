/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE ACCESS user ON DATABASE TYPE RECORD
    SIGNUP ( CREATE user SET email = $email, pass = crypto::argon2::generate($pass) )
    SIGNIN ( SELECT * FROM user WHERE email = $email AND crypto::argon2::compare(pass, $pass) )
    AUTHENTICATE {
        IF type::record("token", $token.jti).revoked = true {
            THROW "This token has been revoked";
        };
        INSERT INTO token { id: $token.jti, exp: $token.exp, revoked: false };
        CREATE audit CONTENT { token: $token.jti, time: time::now() };
        RETURN $auth;
    }
    DURATION FOR TOKEN 30d, FOR SESSION 1h
;
