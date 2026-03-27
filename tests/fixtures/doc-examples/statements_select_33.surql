-- Fails
SELECT * FROM ONLY table_name;
-- Succeeds
SELECT * FROM ONLY table_name LIMIT 1;
