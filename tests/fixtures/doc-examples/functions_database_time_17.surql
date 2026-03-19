/**[test]

[[test.results]]
value = "d'2021-11-04T00:00:00Z'"

*/

RETURN time::round(d"2021-11-01T08:30:17+00:00", 1w);

-- d'2021-11-04T00:00:00Z'
