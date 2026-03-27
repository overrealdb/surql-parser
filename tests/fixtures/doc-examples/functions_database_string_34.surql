/**[test]

[[test.results]]
value = "true"

[[test.results]]
value = "false"

*/

RETURN string::is_datetime("2015-09-05 23:56:04", "%Y-%m-%d %H:%M:%S");
-- true

RETURN string::is_datetime("1970-01-01", "%Y-%m-%d %H:%M:%S");
-- false
