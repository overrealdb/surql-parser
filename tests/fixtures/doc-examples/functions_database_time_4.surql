/**[test]

[[test.results]]
value = "d'2021-10-28T00:00:00Z'"

*/

RETURN time::floor(d"2021-11-01T08:30:17+00:00", 1w);

-- d'2021-10-28T00:00:00Z'
