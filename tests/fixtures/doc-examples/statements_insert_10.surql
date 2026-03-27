/**[test]

[[test.results]]
value = "[NONE]"

*/

-- Before insert will always return NONE as it is the same as the record being inserted
INSERT INTO company {
	name: 'SurrealDB',
	founded: "2021-09-10",
	founders: [person:tobie, person:jaime],
	tags: ['big data', 'database']
} RETURN BEFORE;
