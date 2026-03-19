/**[test]

[[test.results]]
value = "[{ id: person:1 }, { id: person:2 }, { id: person:3 }]"

[[test.results]]
value = "[{ id: likes:object, in: person:1, out: person:2 }]"

[[test.results]]
value = "[{ id: likes:array, in: person:1, out: person:2 }, { id: likes:array_two, in: person:2, out: person:3 }]"

[[test.results]]
value = "[{ id: likes:values, in: person:1, out: person:2 }]"

[[test.results]]
value = "[[likes:array, likes:object, likes:values], [likes:array_two], []]"

*/

-- Insert records into the person table
INSERT INTO person [
	{ id: 1 },
	{ id: 2 },
	{ id: 3 },
];
-- Insert a single relation
INSERT RELATION INTO likes {
	in: person:1,
	id: 'object',
	out: person:2,
};

-- Insert multiple relations
INSERT RELATION INTO likes [
	{
		in: person:1,
		id: 'array',
		out: person:2,
	},
	{
		in: person:2,
		id: 'array_two',
		out: person:3,
	}
];

-- Insert a relation and return the value of the likes field
INSERT RELATION INTO likes (in, id, out)
	VALUES (person:1, 'values', person:2);

-- Select the value of the likes field
SELECT VALUE ->likes FROM person;
