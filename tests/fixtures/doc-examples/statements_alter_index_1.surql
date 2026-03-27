/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

*/

-- Will show up as DEFINE TABLE user TYPE ANY SCHEMAFULL PERMISSIONS NONE
DEFINE TABLE user SCHEMAFULL;

-- Now defined as DEFINE TABLE user TYPE ANY SCHEMAFULL PERMISSIONS FULL
ALTER TABLE user PERMISSIONS FOR create FULL;
