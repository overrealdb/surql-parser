IF $access = "admin" { SELECT * FROM account }
ELSE IF $access = "user" { SELECT * FROM $auth.account }
ELSE {}
