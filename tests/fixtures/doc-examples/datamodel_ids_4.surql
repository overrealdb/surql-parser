/**[test]

[[test.results]]
value = "[{ author: person:tobie, id: article:⟨8424486b-85b3-4448-ac8d-5d51083391c7⟩, time: d'2025-10-03T01:06:17.050210Z' }]"

[[test.results]]
value = "[{ author: person:⟨Lech_Wałęsa⟩, id: ⟨Artykuł⟩:100 }]"

*/

CREATE article:`8424486b-85b3-4448-ac8d-5d51083391c7` SET
    time = time::now(),
    author = person:tobie;

CREATE `Artykuł`:100 SET
    author = person:`Lech_Wałęsa`;
