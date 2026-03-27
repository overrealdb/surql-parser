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
value = "[{ email: 'Tobie.Hitchcock@surrealdb.com', firstName: 'Tobie', id: user:9hizvimsgva1rosul33j, lastName: 'Hitchcock' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ email: 'Tobie.Hitchcock@surrealdb.com', firstName: 'Tobie', id: user:9hizvimsgva1rosul33j, lastName: 'Hitchcock' }]"
skip-record-id-key = true

*/

-- Create schemafull user table.
DEFINE TABLE user SCHEMAFULL;

-- Define some fields.
DEFINE FIELD firstName ON TABLE user TYPE string;
DEFINE FIELD lastName ON TABLE user TYPE string;
DEFINE FIELD email ON TABLE user TYPE string
  ASSERT string::is_email($value);
DEFINE INDEX userEmailIndex ON TABLE user COLUMNS email UNIQUE;

-- SEE IT IN ACTION
-- 1: Add a user with all required fields and an undefined one, 'photoURI'.
CREATE user CONTENT {
    firstName: 'Tobie',
    lastName: 'Hitchcock',
    email: 'Tobie.Hitchcock@surrealdb.com',
    photoURI: 'photo/yxCFi22Jw2.webp'
};
-- 2: Statement will not fail but photoURI will be ignored as it is not a
--    defined field.

-- 3: Query the data
SELECT * FROM user;
