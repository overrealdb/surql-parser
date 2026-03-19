/**[test]

[[test.results]]
value = "[{ id: country:canada }]"

[[test.results]]
value = "[{ id: province:bc }, { id: province:alberta }]"

[[test.results]]
value = "[{ id: city:vancouver }, { id: city:victoria }, { id: city:edmonton }, { id: city:calgary }]"

[[test.results]]
value = "[{ id: in:5bgf21urlmudhbv87weh, in: city:vancouver, out: province:bc }, { id: in:nt3jd2nxdf28t1eeg7uz, in: city:victoria, out: province:bc }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: in:6iqjlca9vrnrcotwfb0a, in: city:edmonton, out: province:alberta }, { id: in:56xr5okh0jbmd5d34r0a, in: city:calgary, out: province:alberta }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: in:gr88pyx3de7nrsm63uo2, in: province:bc, out: country:canada }, { id: in:fidjczbmblc01syn4gzd, in: province:alberta, out: country:canada }]"
skip-record-id-key = true

*/

CREATE country:canada;
CREATE province:bc, province:alberta;
CREATE city:vancouver, city:victoria, city:edmonton, city:calgary;

RELATE [city:vancouver, city:victoria]->in->province:bc;
RELATE [city:edmonton, city:calgary]->in->province:alberta;
RELATE [province:bc, province:alberta]->in->country:canada;
