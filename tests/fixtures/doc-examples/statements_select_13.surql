-- Split the results by each value in a nested array
SELECT * FROM country SPLIT locations.cities;

-- Filter the result of a subquery
SELECT * FROM (SELECT * FROM person SPLIT loggedin) WHERE loggedin > '2023-05-01';
