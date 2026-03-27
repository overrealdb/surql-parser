/**[test]

[[test.results]]
value = "[{ id: person:one }, { id: person:two }, { id: person:three }]"

[[test.results]]
value = "[{ has_been_friends: true, has_dated: true, has_hated: true, id: knows:i7yz3l6dcje9it8qbuwo, in: person:one, married_to: true, out: person:two }]"
skip-record-id-key = true

[[test.results]]
value = "[{ has_dated: true, id: knows:0xfnkabcg0um428rtj2o, in: person:one, out: person:three, same_high_school: true }]"
skip-record-id-key = true

*/

CREATE person:one, person:two, person:three;
RELATE person:one->knows->person:two SET
    has_been_friends = true,
    has_dated = true,
    has_hated = true,
    married_to = true;

RELATE person:one->knows->person:three SET
    same_high_school = true,
    has_dated = true;
