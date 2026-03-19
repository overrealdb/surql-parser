-- Record IDs are unique so guaranteed to be no more than 1
SELECT * FROM ONLY person:jamie;

-- Error because no guarantee that this will return a single record
SELECT * FROM ONLY person WHERE name = "Jaime";

-- Add `LIMIT 1` to ensure that only up to one record will be returned
SELECT * FROM ONLY person WHERE name = "Jaime" LIMIT 1;
