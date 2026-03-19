/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "{ events: {  }, fields: { name: 'DEFINE FIELD name ON user TYPE string PERMISSIONS FULL' }, indexes: {  }, lives: {  }, tables: {  } }"

[[test.results]]
value = "{ events: [], fields: [{ flex: false, kind: 'string', name: 'name', permissions: { create: true, select: true, update: true }, readonly: false, what: 'user' }], indexes: [], lives: [], tables: [] }"

*/

DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD name ON user TYPE STRING;

INFO FOR TABLE user;
INFO FOR TABLE user STRUCTURE;
