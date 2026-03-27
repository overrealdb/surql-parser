/**[test]

[[test.results]]
value = "[]"

*/

-- Insert a record and return nothing
INSERT INTO company {
	name: 'SurrealDB',
	founded: "2021-09-10",
	founders: [person:tobie, person:jaime],
	tags: ['big data', 'database']
} RETURN NONE;
