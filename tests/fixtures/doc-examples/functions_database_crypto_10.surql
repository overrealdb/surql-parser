LET $hash = "$pbkdf2-sha256$i=10000,l=32$DBURRPJODKEt0IId1Lqe+w$Ve8Z00mibHDSKLbyKTceEBBcDpGoK0AEUl7QzDTIec4";
LET $pass = "this is a strong password";
RETURN crypto::pbkdf2::compare($hash, $pass);


true
