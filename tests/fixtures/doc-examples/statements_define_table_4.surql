/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ email: 'Tobie.Hitchcock@surrealdb.com', firstName: 'Tobie', id: user:tobie, lastName: 'Hitchcock', photoURI: 'photo/yxCFi22Jw2.webp' }]"

[[test.results]]
error = ""Found 'Jamie.Hitchcock' for field `email`, with record `user:jaime`, but field must conform to: string::is_email($value)""

*/

-- Create schemaless user table.
DEFINE TABLE user SCHEMALESS;

-- Define some fields.
DEFINE FIELD firstName ON TABLE user TYPE string;
DEFINE FIELD lastName ON TABLE user TYPE string;
DEFINE FIELD email ON TABLE user TYPE string
  ASSERT string::is_email($value);
DEFINE INDEX userEmailIndex ON TABLE user COLUMNS email UNIQUE;

-- SEE IT IN ACTION - Example 1
-- 1: Add a user with all required fields and an undefined one.
CREATE user:tobie SET firstName = 'Tobie', lastName = 'Hitchcock', email = 'Tobie.Hitchcock@surrealdb.com', photoURI = 'photo/yxCFi22Jw2.webp';
-- 2: Statement will succeed because user is a SCHEMALESS table.

-- SEE IT IN ACTION - Example 2
-- 1: Add a user with an invalid email address and include a new field that was never defined.
CREATE user:jaime SET firstName = 'Jamie', lastName = 'Hitchcock', email = 'Jamie.Hitchcock', photoURI = 'photo/yxCFi22Jw2.webp';
-- 2: Statement will fail because the value for email was not valid.
