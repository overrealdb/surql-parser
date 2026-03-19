/**[test]

[[test.results]]
value = "[{ id: person:you, knows: [person:friend1, person:friend2] }]"

[[test.results]]
value = "[{ id: person:friend1, knows: [person:friend2] }]"

[[test.results]]
value = "[{ id: person:friend2, knows: [person:acquaintance1, person:acquaintance2, person:acquaintance3] }]"

[[test.results]]
value = "[{ id: person:acquaintance1 }, { id: person:acquaintance2 }, { id: person:star }]"

[[test.results]]
value = "[{ id: person:acquaintance3, knows: [person:star] }]"

[[test.results]]
value = "[person:friend2, person:acquaintance3, person:star]"

[[test.results]]
value = "[[person:friend2, person:acquaintance1], [person:friend2, person:acquaintance2], [person:friend1, person:friend2, person:acquaintance1], [person:friend1, person:friend2, person:acquaintance2], [person:friend2, person:acquaintance3, person:star], [person:friend1, person:friend2, person:acquaintance3, person:star]]"

[[test.results]]
value = "[person:friend1, person:friend2, person:acquaintance1, person:acquaintance2, person:acquaintance3, person:star]"

*/

CREATE person:you SET knows = [person:friend1, person:friend2];
CREATE person:friend1 SET knows = [person:friend2];
CREATE person:friend2 SET knows = [person:acquaintance1, person:acquaintance2, person:acquaintance3];
CREATE person:acquaintance1, person:acquaintance2, person:star;
CREATE person:acquaintance3 SET knows = [person:star];

person:you.{..+shortest=person:star}.knows;
person:you.{..+path}.knows;
person:you.{..+collect}.knows;
