/**[test]

[[test.results]]
value = "[{ founded: '2021-09-10', founders: [person:tobie, person:jaime], id: company:yzc8d07g376x3qddxmdn, name: 'SurrealDB', tags: ['big data', 'database'] }]"
skip-record-id-key = true

*/

-- Return the record after creation
INSERT INTO company {
	name: 'SurrealDB',
	founded: "2021-09-10",
	founders: [person:tobie, person:jaime],
	tags: ['big data', 'database']
} RETURN AFTER;
