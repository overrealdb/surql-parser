/**[test]

[[test.results]]
value = "NONE"

*/

-- Create an USER and overwrite if it already exists
DEFINE USER OVERWRITE example ON ROOT PASSWORD "example" ROLES OWNER;
