/**[test]

[[test.results]]
value = "[]"

*/

INSERT INTO recordings_san_francisco (SELECT * FROM temperature WHERE city = 'San Francisco');
