/**[test]

[[test.results]]
value = "[{ id: person:you, name: 'you' }, { id: person:friend1, name: 'friend1' }, { id: person:friend2, name: 'friend2' }, { id: person:acquaintance1, name: 'acquaintance1' }, { id: person:acquaintance2, name: 'acquaintance2' }, { id: person:acquaintance3, name: 'acquaintance3' }, { id: person:star, name: 'star' }]"

[[test.results]]
value = "[{ id: knows:i8twxf1ghl053391z54j, in: person:you, out: person:friend1 }, { id: knows:071ho0yn3tol4kgwu0fn, in: person:you, out: person:friend2 }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: knows:8fnyab57dotlu2572uc8, in: person:friend1, out: person:friend2 }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: knows:0u34elv08i9ci7igim0f, in: person:friend2, out: person:acquaintance1 }, { id: knows:hgosmpddbv6nuohsgj8p, in: person:friend2, out: person:acquaintance2 }, { id: knows:c04oc5168ubocheihbtu, in: person:friend2, out: person:acquaintance3 }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: knows:d0s3lq6ynweqaryh567o, in: person:acquaintance3, out: person:star }]"
skip-record-id-key = true

*/

CREATE 
	person:you, 
	person:friend1, person:friend2, 
	person:acquaintance1, person:acquaintance2, person:acquaintance3, 
	person:star;

-- You have two friends
RELATE person:you->knows->[person:friend1, person:friend2];
-- The first friend is shy and only knows one other person
RELATE person:friend1->knows->person:friend2;
-- The second friend is very social and knows many people you barely know
RELATE person:friend2->knows->[person:acquaintance1, person:acquaintance2, person:acquaintance3];
-- One of those people knows the movie star
RELATE person:acquaintance3->knows->person:star;
