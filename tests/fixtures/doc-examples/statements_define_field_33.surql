/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ age: 6, extra_self: { age: 6, id: person:one, name: 'Little person' }, id: person:one, name: 'Little person' }]"

*/

DEFINE FIELD extra_self ON TABLE person VALUE $this;
CREATE person:one SET name = "Little person", age = 6;
