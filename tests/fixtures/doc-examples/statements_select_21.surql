-- Order records randomly
SELECT * FROM user ORDER BY rand();

-- Order records descending by a single field
SELECT * FROM song ORDER BY rating DESC;

-- Order records by multiple fields independently
SELECT * FROM song ORDER BY artist ASC, rating DESC;

-- Order text fields with Unicode collation
SELECT * FROM article ORDER BY title COLLATE ASC;

-- Order text fields with which include numeric values
SELECT * FROM article ORDER BY title NUMERIC ASC;
