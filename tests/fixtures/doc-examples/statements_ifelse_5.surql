RETURN
    IF $access = "admin" { (SELECT * FROM account) }
    ELSE IF $access = "user"  { (SELECT * FROM $auth.account) }
    ELSE { THROW "Access method hasn't been defined!" };
