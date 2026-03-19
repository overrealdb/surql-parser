/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

*/

DEFINE TABLE product SCHEMAFULL;
-- Set a default value of 123.456 for the primary field
DEFINE FIELD primary ON product TYPE number DEFAULT ALWAYS 123.456;
