/**[test]

[[test.results]]
value = "[{ age: 18, id: person:one }]"

[[test.results]]
value = "[{ age: 40, id: person:two }]"

[[test.results]]
value = "[{ age: 18, id: person:three }]"

[[test.results]]
value = "{ age: 18, id: person:one, same_age: [{ age: 18, id: person:one }, { age: 18, id: person:three }] }"

[[test.results]]
value = "{ age: 18, id: person:one, same_age: [{ age: 18, id: person:three }] }"

*/

CREATE person:one SET age = 18;
CREATE person:two SET age = 40;
CREATE person:three SET age = 18;

-- Find all 'person' records of the same age as 'person:one'
-- Here 'person:one' is the $parent of the inner operation
person:one.{
	id,
    age,
    same_age: SELECT * FROM person WHERE age = $parent.age
};

-- Now use array::complement to filter out the 'person:one' current record,
-- which is the parameter $this
person:one.{
	id,
    age,
    same_age: array::complement(SELECT * FROM person WHERE age = $parent.age, [$this])
};
