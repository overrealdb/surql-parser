-- All queries lead to `person:artistotle` twice,
-- via different paths and thus different field names
-- and/or structure

-- Directly from the `wrote` table
SELECT in FROM wrote;

-- From a single `person` record
SELECT ->wrote.in FROM person;
SELECT ->wrote<-person FROM person;

-- From two `article` records
SELECT <-wrote.in FROM article;
SELECT <-wrote<-person FROM article;
