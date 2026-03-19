/**[test]

[[test.results]]
value = "[{ id: person:two }]"

[[test.results]]
value = "[{ children: [person:two], id: person:one }]"

[[test.results]]
value = "[{ id: person:one }]"

[[test.results]]
value = "[{ id: person:one }, { id: person:two }]"

*/

CREATE person:two;
CREATE person:one SET children = [person:two];
UPDATE person:one SET children = NONE;
SELECT * FROM person;
