SELECT out FROM wrote;

SELECT ->wrote.out FROM person;
SELECT ->wrote->article FROM person;

SELECT <-wrote.out FROM article;
SELECT <-wrote->article FROM article;
