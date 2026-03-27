/**[test]

[[test.results]]
value = "[{ id: person:one }, { id: person:two }]"

[[test.results]]
value = "[{ id: sees:1906dfpey5ho324ot0if, in: person:one, out: person:two }]"

[[test.results]]
value = "[{ id: person:two, is_seen_by: [person:one] }]"

*/

CREATE person:one, person:two;
RELATE person:one->sees->person:two;

(SELECT 
  id, 
  <-sees<-person AS is_seen_by
FROM person)
    .filter(|$person| $person.is_seen_by);
