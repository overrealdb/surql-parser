-- Order records randomly
SELECT * FROM <table> ORDER BY rand();

-- Order records descending by a single field
SELECT * FROM <table> ORDER BY <field> DESC;

-- Order records by multiple fields independently
SELECT * FROM <table> ORDER BY <field> ASC, <field2> DESC;

-- Order text fields with lexical collation instead of Unicode order
SELECT * FROM <table> ORDER BY <field> COLLATE ASC;

-- Order text fields with which include numeric values
SELECT * FROM <table> ORDER BY <field> NUMERIC ASC;

-- COLLATE and NUMERIC can be used together
SELECT * FROM <table> ORDER BY <field> COLLATE NUMERIC ASC;
