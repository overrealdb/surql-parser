/**[test]

[[test.results]]
value = "[{ id: person:one }, { id: person:two }, { id: person:three }, { id: person:four }]"

[[test.results]]
value = "[{ id: likes:1, in: person:one, out: person:two }]"

[[test.results]]
value = "[{ id: likes:2, in: person:one, out: person:three }]"

[[test.results]]
value = "[{ id: likes:3, in: person:one, out: person:four }]"

[[test.results]]
value = "[person:three, person:four]"

*/

CREATE person:one, person:two, person:three, person:four;

RELATE person:one->likes:1->person:two;
RELATE person:one->likes:2->person:three;
RELATE person:one->likes:3->person:four;

person:one->likes:2..=4->person;
