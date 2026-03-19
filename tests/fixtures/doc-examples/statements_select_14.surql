-- Group records by a single field
SELECT country FROM user GROUP BY country;

-- Group results by a nested field
SELECT settings.published FROM article GROUP BY settings.published;

-- Group results by multiple fields
SELECT gender, country, city FROM person GROUP BY gender, country, city;

-- Use an aggregate function to select unique values from a nested array across an entire table
SELECT array::group(tags) AS tags FROM article GROUP ALL;
