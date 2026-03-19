/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE ACCESS user ON DATABASE TYPE JWT
ALGORITHM HS512 KEY "sNSYneezcr8kqphfOC6NwwraUHJCVAt0XjsRSNmssBaBRh3WyMa9TRfq8ST7fsU2H2kGiOpU4GbAF1bCiXmM1b3JGgleBzz7rsrz6VvYEM4q3CLkcO8CMBIlhwhzWmy8"
AUTHENTICATE {
  IF $token.iss != "surrealdb-test" { THROW "Invalid token issuer" };
  IF type::is_array($token.aud) {
    IF "surrealdb-test" NOT IN $token.aud { THROW "Invalid token audience" }
  } ELSE {
    IF $token.aud IS NOT "surrealdb-test" { THROW "Invalid token audience" }
  };
}
DURATION FOR SESSION 2h;
