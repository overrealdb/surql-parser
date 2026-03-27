/**[test]

[[test.results]]
value = "[{ founded: '2021-09-10', founders: [person:tobie, person:jaime], id: company:5fc7p7d0kiirdx7accfn, name: 'SurrealDB', tags: ['big data', 'database'] }]"
skip-record-id-key = true

*/

INSERT INTO company {
	name: 'SurrealDB',
	founded: "2021-09-10",
	founders: [person:tobie, person:jaime],
	tags: ['big data', 'database']
};
