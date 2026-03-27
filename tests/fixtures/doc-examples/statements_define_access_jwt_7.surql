/**[test]

[[test.results]]
value = "NONE"

*/

-- Create a JWT access method for the example database if it does not already exist
DEFINE ACCESS IF NOT EXISTS example ON DATABASE TYPE JWT ALGORITHM HS512 KEY
"sNSYneezcr8kqphfOC6NwwraUHJCVAt0XjsRSNmssBaBRh3WyMa9TRfq8ST7fsU2H2kGiOpU4GbAF1bCiXmM1b3JGgleBzz7rsrz6VvYEM4q3CLkcO8CMBIlhwhzWmy8";
