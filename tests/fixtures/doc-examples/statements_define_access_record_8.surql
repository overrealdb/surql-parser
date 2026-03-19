/**[test]

[[test.results]]
value = "NONE"

*/

-- Create a RECORD access method for the example database if it does not already exist
DEFINE ACCESS IF NOT EXISTS example ON DATABASE TYPE RECORD;
