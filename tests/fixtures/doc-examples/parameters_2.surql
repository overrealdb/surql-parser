/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
error = "'Thrown error: Database record `person:billy` already exists'"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "'Thrown error: Database record `person:billy` already exists'"

*/

LET $billy = CREATE ONLY person:billy SET name = "Billy";
-- Fails as `person:billy` already exists
CREATE person CONTENT $billy;

USE NAMESPACE other_namespace;
USE DATABASE other_database;
-- Succeeds as `person:billy` does not yet exist in this namespace and database
CREATE person CONTENT $billy;
