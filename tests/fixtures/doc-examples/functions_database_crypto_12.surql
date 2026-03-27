LET $hash = "$scrypt$ln=15,r=8,p=1$8gl7bipl0FELTy46YJOBrw$eRcS1qR22GI8VHo58WOXn9JyfDivGo5yTJFvpDyivuw";
LET $pass = "this is a strong password";
RETURN crypto::scrypt::compare($hash, $pass);


true
