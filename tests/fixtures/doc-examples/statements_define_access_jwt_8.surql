/**[test]

[[test.results]]
value = "NONE"

*/

-- Create a JWT access method for the example database and overwrite it if it already exists
DEFINE ACCESS OVERWRITE example ON DATABASE TYPE JWT ALGORITHM HS512 KEY 'secret';
