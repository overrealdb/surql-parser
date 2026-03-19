-- Select nested objects/values
SELECT address.city FROM person;

-- Select all nested array values
-- note the .* syntax works to select everything from an array or object-like values
SELECT address.*.coordinates AS coordinates FROM person;
-- Equivalent to
SELECT address.coordinates AS coordinates FROM person;

-- Select one item from an array
SELECT address.coordinates[0] AS latitude FROM person;

-- Select unique values from an array
SELECT array::distinct(tags) FROM article;

-- Select unique values from a nested array across an entire table
SELECT array::group(tags) AS tags FROM article GROUP ALL;

-- Use mathematical calculations in a select expression
SELECT
	(( celsius * 1.8 ) + 32) AS fahrenheit
	FROM temperature;

-- Return boolean expressions with an alias
SELECT rating >= 4 as positive FROM review;

-- Select manually generated object structure
SELECT
	{ weekly: false, monthly: true } AS `marketing settings`
FROM user;

-- Select filtered nested array values
SELECT address[WHERE active = true] FROM person;

-- Select a person who has reacted to a post using a celebration
-- Path can be conceptualized as:
-- person->(reacted_to WHERE type='celebrate')->post
SELECT * FROM person WHERE ->(reacted_to WHERE type='celebrate')->post;

-- Select a remote field from connected out graph edges
SELECT ->likes->friend.name AS friends FROM person:tobie;

-- Use the result of a subquery as a returned field
SELECT *, (SELECT * FROM events WHERE type = 'activity' LIMIT 5) AS history FROM user;

-- Restructure objects in a select expression after `.` operator
SELECT address.{city, country} FROM person;
