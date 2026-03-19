/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

*/

-- Specify the namespace for the token
USE NS abcum;

-- Set the name of the token
DEFINE ACCESS token_name
  -- Use this token provider for namespace authentication
  ON NAMESPACE
  -- Specify the type of access being defined
  TYPE JWT
  -- Specify the cryptographic signature algorithm used to verify the token
  ALGORITHM HS512
  -- Specify the symmetric key used to sign and verify the authenticity of the token
  KEY "sNSYneezcr8kqphfOC6NwwraUHJCVAt0XjsRSNmssBaBRh3WyMa9TRfq8ST7fsU2H2kGiOpU4GbAF1bCiXmM1b3JGgleBzz7rsrz6VvYEM4q3CLkcO8CMBIlhwhzWmy8"
;
