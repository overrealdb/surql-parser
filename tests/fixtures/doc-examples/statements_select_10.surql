-- Selects all records from both 'user' and 'admin' tables.
SELECT * FROM user, admin;

-- Selects all records from the table named in the variable '$table',
-- but only if the 'admin' field of those records is true.
-- Equivalent to 'SELECT * FROM user WHERE admin = true'.
LET $table = "user";
SELECT * FROM type::table($table) WHERE admin = true;

-- Selects a single record from:
-- * the table named in the variable '$table',
-- * and the identifier named in the variable '$id'.
-- This query is equivalent to 'SELECT * FROM user:admin'.
LET $table = "user";
LET $id = "admin";
SELECT * FROM type::record($table, $id);

-- Selects all records for specific users 'tobie' and 'jaime',
-- as well as all records for the company 'surrealdb'.
SELECT * FROM user:tobie, user:jaime, company:surrealdb;

-- Selects records from a list of identifiers. The identifiers can be numerical,
-- string, or specific records such as 'person:lrym5gur8hzws72ux5fa'.
SELECT * FROM [3648937, "test", person:lrym5gur8hzws72ux5fa, person:4luro9170uwcv1xrfvby];

-- Selects data from an object that includes a 'person' key,
-- which is associated with a specific person record, and an 'embedded' key set to true.
SELECT * FROM { person: person:lrym5gur8hzws72ux5fa, embedded: true };

-- This command first performs a subquery, which selects all 'user' records and adds a
-- computed 'adult' field that is true if the user's 'age' is 18 or older.
-- The main query then selects all records from this subquery where 'adult' is true.
SELECT * FROM (SELECT age >= 18 AS adult FROM user) WHERE adult = true;
