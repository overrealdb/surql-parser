-- Create 5000 `person` records
CREATE |person:1..5000| RETURN NONE;

-- Set the starting time
LET $now = time::now();
-- Put the output somewhere so it won't clutter the screen
LET $_ = SELECT * FROM person:1..5000;
-- Get the elapsed time
LET $time1 = time::now() - $now;

LET $now = time::now();
LET $_ = SELECT * FROM person WHERE id >= 1 and id <= 5000;
LET $time2 = time::now() - $now;
RETURN [$time1, $time2];
