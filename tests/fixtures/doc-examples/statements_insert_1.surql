/**[test]

[[test.results]]
value = "[{ founded: '2021-09-10', id: company:l6luw1w1lqr7v2loau6a, name: 'SurrealDB' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ founded: '1967-05-03', id: company:racyetsf1x1hbu7d5g5s, name: 'Acme Inc.' }, { founded: '1976-04-01', id: company:x9pi148wprp30jfv5o4q, name: 'Apple Inc.' }]"
skip-record-id-key = true

*/

-- Insert a single record
INSERT INTO
	company (name, founded)
	VALUES  ('SurrealDB', '2021-09-10');

-- Insert multiple records
INSERT INTO
	company (name, founded)
	VALUES  ('Acme Inc.', '1967-05-03'), ('Apple Inc.', '1976-04-01');
