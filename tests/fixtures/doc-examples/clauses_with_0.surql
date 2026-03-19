-- forces the query planner to use the specified index(es):
SELECT * FROM person
WITH INDEX ft_email
WHERE
	email = 'tobie@surrealdb.com' AND
	company = 'SurrealDB';

-- forces the usage of the table iterator
SELECT name FROM person WITH NOINDEX WHERE job = 'engineer' AND gender = 'm';
