-- Checks with current datetime if none is passed
RETURN time::is_leap_year();

RETURN time::is_leap_year(d"1987-06-22T08:30:45Z");
-- false

RETURN time::is_leap_year(d"1988-06-22T08:30:45Z");
-- true

-- Using function via method chaining
RETURN d'2024-09-03T02:33:15.349397Z'.is_leap_year();
-- true
