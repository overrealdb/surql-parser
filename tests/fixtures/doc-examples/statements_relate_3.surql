-- Aristotle's id and the articles he wrote
SELECT id, ->wrote->article FROM person:aristotle;
-- Every `person`'s id and written articles
-- Same output as above as the database has a single `person` record
SELECT id, ->wrote->article FROM person;
-- Directly follow the path from Aristotle to his written articles
RETURN person:aristotle->wrote->article;
