-- These two statements store the result of the subquery in a parameter
-- The subquery returns an array of IDs
LET $person =  (SELECT VALUE id FROM person);
LET $article = (SELECT VALUE id FROM article);

-- This statement creates a relationship record for every combination of Record IDs
-- Such that if we have 10 records each in the person and article table
-- We get 100 records in the wrote edge table (10*10 = 100)
-- In this case it would mean that each article would have 10 authors
RELATE $person->wrote->$article SET time.written = time::now();
