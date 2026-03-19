/**[test]

[[test.results]]
value = "[{ id: some_table:8o6rh2r3ovpmu07elc3r }]"

[[test.results]]
value = "NONE"

[[test.results]]
value = "{ accesses: {  }, analyzers: {  }, apis: {  }, buckets: {  }, configs: {  }, functions: {  }, models: {  }, params: {  }, sequences: {  }, tables: { some_other_table: 'DEFINE TABLE some_other_table TYPE ANY SCHEMALESS PERMISSIONS NONE', some_table: 'DEFINE TABLE some_table TYPE ANY SCHEMALESS PERMISSIONS NONE' }, users: {  } }"

*/

CREATE some_table;
DEFINE TABLE some_other_table;

INFO FOR DB;
