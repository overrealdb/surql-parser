/**[test]

[[test.results]]
value = "NONE"

*/

-- Create the user with an owner role and some example durations
DEFINE USER username ON ROOT PASSWORD '123456' ROLES OWNER DURATION FOR SESSION 15m, FOR TOKEN 5s;
