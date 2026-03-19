/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "{ events: {  }, fields: { friends: 'DEFINE FIELD friends ON person TYPE array<record<person>> PERMISSIONS FULL', "friends.*": 'DEFINE FIELD friends.* ON person TYPE record<person> PERMISSIONS FULL' }, indexes: {  }, lives: {  }, tables: {  } }"

*/

DEFINE FIELD friends ON TABLE person TYPE array<record<person>>;
INFO FOR TABLE person;
