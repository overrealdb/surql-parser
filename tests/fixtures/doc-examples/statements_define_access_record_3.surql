/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE ACCESS token_name ON DATABASE TYPE RECORD WITH JWT
-- Symmetric algorithm with a symmetric key
-- The same key is used to sign and verify
ALGORITHM HS512 KEY "secret";
-- The following clause is implicit:
-- WITH ISSUER ALGORITHM HS512 KEY "secret"
