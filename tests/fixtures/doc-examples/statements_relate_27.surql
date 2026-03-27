SELECT time_written FROM wrote;
SELECT ->wrote.time_written as time_written FROM person;
SELECT <-wrote.time_written as time_written FROM article;
