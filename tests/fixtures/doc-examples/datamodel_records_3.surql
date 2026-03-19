/**[test]

[[test.results]]
value = "[{ friends: [person:tobie, person:simon], id: person:jaime, name: 'Jaime' }]"

[[test.results]]
value = "[{ friends: [person:simon, person:marcus], id: person:tobie, name: 'Tobie' }]"

[[test.results]]
value = "[{ friends: [person:jaime, person:tobie], id: person:simon, name: 'Simon' }]"

[[test.results]]
value = "[{ friends: [person:tobie], id: person:marcus, name: 'Marcus' }]"

*/

CREATE person:jaime SET name = 'Jaime', friends = [person:tobie, person:simon];
CREATE person:tobie SET name = 'Tobie', friends = [person:simon, person:marcus];
CREATE person:simon SET name = 'Simon', friends = [person:jaime, person:tobie];
CREATE person:marcus SET name = 'Marcus', friends = [person:tobie];
