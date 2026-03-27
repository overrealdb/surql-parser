-- Create 3 `person` records
CREATE |person:3|;

LET $now = time::now();

SELECT *, 
  sleep(100ms) AS _, 
  time::now() - $now AS elapsed
FROM person;
