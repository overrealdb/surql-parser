/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE ACCESS user ON DATABASE TYPE RECORD
    WITH JWT ALGORITHM HS512 KEY 'secret'
    AUTHENTICATE {
        IF $auth.id {
            RETURN $auth.id;
        } ELSE IF $token.email {
            RETURN SELECT * FROM user WHERE email = $token.email;
        };
    }
;
