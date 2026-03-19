/**[test]

[[test.results]]
value = "NONE"

*/

-- Create a USER if it does not already exist
DEFINE USER IF NOT EXISTS example ON ROOT PASSWORD "example" ROLES OWNER;
