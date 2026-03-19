/**[test]

[[test.results]]
value = "[{ id: person:hermann_hesse }, { id: person:abigail }, { id: city:calw }, { id: book:demian }]"

[[test.results]]
value = "[{ id: wrote:lj0hszy4vbti435r0gza, in: person:hermann_hesse, out: book:demian, written_in: d'1919-01-01T00:00:00Z' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: born_in:0mdrzu8ubnff42lder2c, in: person:hermann_hesse, out: city:calw }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: likes:8pbzxj55dcad7rbrt55y, in: person:abigail, out: person:hermann_hesse }]"
skip-record-id-key = true

[[test.results]]
value = "[{ what_hesse_did: [{ id: born_in:0mdrzu8ubnff42lder2c, in: person:hermann_hesse, out: city:calw }, { id: wrote:lj0hszy4vbti435r0gza, in: person:hermann_hesse, out: book:demian, written_in: d'1919-01-01T00:00:00Z' }], what_others_did_to_hesse: [{ id: likes:8pbzxj55dcad7rbrt55y, in: person:abigail, out: person:hermann_hesse }] }]"
skip-record-id-key = true

*/

CREATE person:hermann_hesse, person:abigail, city:calw, book:demian;
RELATE person:hermann_hesse->wrote->book:demian SET written_in = d'1919-01-01';
RELATE person:hermann_hesse->born_in->city:calw;
RELATE person:abigail->likes->person:hermann_hesse;

SELECT 
	-- all tables in which the record is at `in`
    ->(?).* AS what_hesse_did,
	-- all tables in which the record is at `out`
    <-(?).* AS what_others_did_to_hesse
FROM person:hermann_hesse;
