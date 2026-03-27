/**[test]

[[test.results]]
value = "NONE"

*/

-- Create an index on the account and email fields of the user table
DEFINE INDEX test ON user FIELDS account, email UNIQUE;
