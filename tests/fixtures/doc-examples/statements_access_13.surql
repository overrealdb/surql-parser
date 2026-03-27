/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE ACCESS account ON DATABASE TYPE RECORD
    SIGNUP ({
    IF $email = "me@me.com" {
        THROW "That's my email!!!"
    } ELSE {
        CREATE user SET email = $email, pass = crypto::argon2::generate($pass)}   
    })
    SIGNIN ( SELECT * FROM user WHERE email = $email AND crypto::argon2::compare(pass, $pass) )
    DURATION FOR TOKEN 15m, FOR SESSION 12h
;
