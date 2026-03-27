/**[test]

[[test.results]]
value = "[{ id: person:test, name: 'Jaime' }]"

*/

UPSERT person:test SET name = 'Jaime' WHERE name = 'Jaime';
