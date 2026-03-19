/**[test]

[[test.results]]
value = "[{ id: person:39v37mew631umqb63bfq, name: 'Jaime' }]"
skip-record-id-key = true

*/

UPSERT person SET name = 'Jaime' WHERE name = 'Jaime';
