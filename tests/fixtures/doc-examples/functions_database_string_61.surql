/**[test]

[[test.results]]
value = "51"

[[test.results]]
value = "47"

*/

-- returns 51
RETURN string::similarity::fuzzy("DB", "DB");
-- returns 47
RETURN string::similarity::fuzzy("DB", "db");
