DEFINE FIELD name ON person TYPE string;
CREATE person:one SET name = "Billy";
REMOVE FIELD name ON person;

SELECT * FROM person; -- 'name' data is still there
UPDATE person; -- Does nothing
-- [{ id: person:one, name: 'Billy' }]
UPDATE person SET name = NONE; -- Must unset to remove 'name' data
