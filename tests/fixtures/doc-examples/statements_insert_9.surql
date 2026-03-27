/**[test]

[[test.results]]
value = "[[{ op: 'replace', path: '', value: { founded: '2021-09-10', founders: [person:tobie, person:jaime], id: company:tmx6q7chr1vt73t95gx8, name: 'SurrealDB', tags: ['big data', 'database'] } }]]"
skip-record-id-key = true

*/

-- Insert a record and return the diff
INSERT INTO company {
	name: 'SurrealDB',
	founded: "2021-09-10",
	founders: [person:tobie, person:jaime],
	tags: ['big data', 'database']
} RETURN DIFF;
