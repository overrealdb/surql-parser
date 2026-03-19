/**[test]

[[test.results]]
value = "[{ friends: [person:jaime, person:micha], id: person:tobie, name: 'Tobie' }, { friends: [person:mary], id: person:jaime, name: 'Jaime' }, { friends: [person:john], id: person:micha, name: 'Micha' }, { id: person:john, name: 'John' }, { id: person:mary, name: 'Mary' }, { id: person:tim, name: 'Tim' }]"

[[test.results]]
value = "[{ id: knows:1, in: person:tobie, out: person:jaime }, { id: knows:2, in: person:tobie, out: person:micha }, { id: knows:3, in: person:micha, out: person:john }, { id: knows:4, in: person:jaime, out: person:mary }, { id: knows:5, in: person:mary, out: person:tim }]"

[[test.results]]
value = "['Tim']"

[[test.results]]
value = "['Tim']"

[[test.results]]
value = "['Tim']"

[[test.results]]
value = "['Tim']"

[[test.results]]
value = "{ connections: [{ connections: [{ connections: [{ connections: [], id: person:tim, name: 'Tim' }], id: person:mary, name: 'Mary' }], id: person:jaime, name: 'Jaime' }, { connections: [{ connections: [], id: person:john, name: 'John' }], id: person:micha, name: 'Micha' }], id: person:tobie, name: 'Tobie' }"

[[test.results]]
value = "[{ names_2nds: ['Tim'] }, { names_2nds: [] }, { names_2nds: ['Tim'] }, { names_2nds: ['John'] }, { names_2nds: [] }, { names_2nds: ['Tim'] }]]"

[[test.results]]
value = ""

*/

INSERT INTO person [
	{ id: person:tobie, name: 'Tobie', friends: [person:jaime, person:micha] },
	{ id: person:jaime, name: 'Jaime', friends: [person:mary] },
	{ id: person:micha, name: 'Micha', friends: [person:john] },
	{ id: person:john, name: 'John' },
	{ id: person:mary, name: 'Mary' },
	{ id: person:tim, name: 'Tim' },
];

INSERT RELATION INTO knows [
	{ id: knows:1, in: person:tobie, out: person:jaime },
	{ id: knows:2, in: person:tobie, out: person:micha },
	{ id: knows:3, in: person:micha, out: person:john },
	{ id: knows:4, in: person:jaime, out: person:mary },
	{ id: knows:5, in: person:mary, out: person:tim },
];

-- Any depth
person:tobie.{..}(->knows->person).name;

-- Minimum 2, maximum 5 iterations of recursion (or either)
person:tobie.{2..6}(->knows->person).name;
person:tobie.{2..}(->knows->person).name;
person:tobie.{..6}(->knows->person).name;

-- Generate complex recursive tree structures:
-- Fetches connections up to 3 levels deep, 
-- collecting their name, id, and connections along the way
-- 3 levels, because the first iteration is used to collect
-- the details for person:tobie
person:tobie.{..4}.{ id, name, connections: ->knows->person.@ };

-- @ is a shortcut to the current document, and acts as a shorthand to start an idiom path.
-- The "." can optionally be omitted
SELECT @{1..4}(->knows->person).name AS names_2nds FROM person;

-- Recursive idioms work with any idiom parts, not limited to graphs
-- Here, we recursively fetch friends and then collect their names
person:tobie.{1..5}(.friend).name;
