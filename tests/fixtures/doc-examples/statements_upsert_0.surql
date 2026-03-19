/**[test]

[[test.results]]
value = "[{ id: person:5014eu5j3ysw6ifsmk8v, name: 'Billy' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: person:89m30prs5ph4azxluwww, name: 'Bobby' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: person:5014eu5j3ysw6ifsmk8v, name: 'Billy' }, { id: person:89m30prs5ph4azxluwww, name: 'Bobby' }]"
skip-record-id-key = true

*/

UPSERT person SET name = 'Billy';
UPSERT person SET name = 'Bobby';
SELECT * FROM person;
