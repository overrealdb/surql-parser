/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ id: post:test, tags: [] }]"

[[test.results]]
value = "[{ id: post:test, tags: [{ color: 'red', name: 'test' }] }]"

[[test.results]]
value = "[{ id: post:test, tags: [{ color: 'red', name: 'test' }, { color: 'blue', name: 'test' }] }]"

*/

DEFINE TABLE post SCHEMAFULL;
DEFINE FIELD tags ON post TYPE array<object> DEFAULT ALWAYS [];
DEFINE FIELD tags.*.color ON post TYPE string DEFAULT ALWAYS 'red';
DEFINE FIELD tags.*.name ON post TYPE string;
--
CREATE post:test;
UPSERT post:test SET tags += { name: 'test' };
UPSERT post:test SET tags += { name: 'test', color: 'blue' };
