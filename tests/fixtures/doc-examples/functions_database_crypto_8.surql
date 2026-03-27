LET $hash = "$2b$12$OD7hrr1Hycyk8NUwOekYY.cogCICpUnwNvDZ9NiC1qCPHzpVAQ9BO";
LET $pass = "this is a strong password";
RETURN crypto::bcrypt::compare($hash, $pass);

true
