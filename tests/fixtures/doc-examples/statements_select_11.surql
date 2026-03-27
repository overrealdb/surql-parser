-- Simple conditional filtering
SELECT * FROM article WHERE published = true;

-- Conditional filtering based on graph edges
SELECT * FROM profile WHERE count(->experience->organisation) > 3;

-- Conditional filtering based on graph edge properties
SELECT * FROM person WHERE ->(reaction WHERE type='celebrate')->post;

-- Conditional filtering with boolean logic
SELECT * FROM user WHERE (admin AND active) OR owner = true;

-- Select filtered nested array values
SELECT address[WHERE active = true] FROM person;

-- Select names for 'person' records as long as 'name' is present
-- and not an empty string ""
SELECT name FROM person WHERE name;
