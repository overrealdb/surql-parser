-- Select all the review information
-- and the artist's email from the artist table
SELECT *, artist.email FROM review FETCH artist;

-- Select all the article information
-- only if the author's age (from the author table) is under 30.
SELECT * FROM article WHERE author.age < 30 FETCH author;
