/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ email: 'Tobie.Hitchcock@surrealdb.com', firstName: 'Tobie', id: user:tobie, lastName: 'Hitchcock' }]""
skip-record-id-key = true

[[test.results]]
value = "[{ email: 'c@d.com', firstName: 'A', id: user:abc, lastName: 'B' }]"

[[test.results]]
value = "[{ email: 'g@h.com', firstName: 'E', id: user:efg, lastName: 'F' }]""

[[test.results]]
value = "[{ id: document:SurrealDB_whitepaper, name: 'some messaging queue' }]"

[[test.results]]
value = "[{ id: acl:8xq08j6rackxaydwu5u9, permissions: ['create', 'write', 'read'], resource: document:SurrealDB_whitepaper, user: user:tobie }]"

[[test.results]]
value = "[{ id: acl:lov6b2y9jf7m05jsjzrc, permissions: ['read', 'delete'], resource: document:SurrealDB_whitepaper, user: user:abc }]"

[[test.results]]
error = ""Found [] for field `permissions`, with record `acl:invalid`, but field must conform to: (array::len($value) > 0) AND ($value ALLINSIDE ['create', 'read', 'write', 'delete'])""

[[test.results]]
error = ""Found ['all'] for field `permissions`, with record `acl:also_invalid`, but field must conform to: (array::len($value) > 0) AND ($value ALLINSIDE ['create', 'read', 'write', 'delete'])""

*/

-- An ACL can be applied to any kind of resource (record)
DEFINE FIELD resource ON TABLE acl TYPE record;
-- We associate the acl with a user using record<user>
DEFINE FIELD user ON TABLE acl TYPE record<user>;

-- The permissions for the user+resource will be stored in an array.
DEFINE FIELD permissions ON TABLE acl TYPE array
  -- The array must not be empty because at least one permission is required to make a valid ACL
  -- The items in the array must also be restricted to specific permissions
  ASSERT
      array::len($value) > 0
      AND $value ALLINSIDE ["create", "read", "write", "delete"];

-- SEE IT IN ACTION
-- 1: Add users
CREATE user:tobie SET firstName = 'Tobie', lastName = 'Hitchcock',
  email = 'Tobie.Hitchcock@surrealdb.com';
CREATE user:abc SET firstName = 'A', lastName = 'B',
  email = 'c@d.com';
CREATE user:efg SET firstName = 'E', lastName = 'F',
  email = 'g@h.com';

-- 2: Create a resource
CREATE document:SurrealDB_whitepaper SET
  name = "some messaging queue";

-- 3: Associate with ACL
CREATE acl SET user = user:tobie, resource = document:SurrealDB_whitepaper, permissions = ["create", "write", "read"];
CREATE acl SET user = user:abc, resource = document:SurrealDB_whitepaper, permissions = ["read", "delete"];

-- Test Asserts using failure examples
-- A: Create ACL without permissions field
CREATE acl:invalid SET
  user = user:efg,
  permissions = [], # FAIL - permissions must not be empty
  resource = document:SurrealDB_whitepaper;
-- B: Create acl with invalid permisson
CREATE acl:also_invalid SET
  user = user:efg,
  permissions = ["all"], # FAIL - This value is not allowed in the array
  resource = document:SurrealDB_whitepaper;
