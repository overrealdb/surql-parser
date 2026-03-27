-- Update all records which match the condition that `company` is not equal to "SurrealDB"
UPDATE person SET skills += "System design" WHERE company != "SurrealDB";
