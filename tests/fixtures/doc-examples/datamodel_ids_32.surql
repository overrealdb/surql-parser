/**[test]

[[test.results]]
value = "[{ data: { data: 'for', demonstration: 'purposes', some: 'demo' }, id: person:1 }]"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ id: person:1 }]"

*/

CREATE person:1 SET data = {
    some: "demo",
    data: "for",
    demonstration: "purposes"
};

LET $record = SELECT id FROM person:1;
SELECT * FROM $record;
