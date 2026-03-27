DEFINE TABLE person SCHEMAFULL;
DEFINE FIELD name ON person TYPE string;
CREATE person:one SET name = "Billy";
REMOVE FIELD name ON person;

SELECT * FROM person; -- 'name' data is still there
UPDATE person; -- Found field 'name', but no such field exists for table 'person'
DEFINE FIELD created_at ON person TYPE datetime; -- Define a new field

-- Works because values matche schema: 'name' is set to NONE, 'created_at' has a datetime value
UPDATE person SET name = NONE, created_at = time::now();
