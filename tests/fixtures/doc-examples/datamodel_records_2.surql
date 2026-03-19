/**[test]

[[test.results]]
value = "[{ email: 'tobie@surrealdb.com', id: person:tobie, name: 'Tobie', opts: { enabled: true } }]"

[[test.results]]
value = "{ email: 'tobie@surrealdb.com', id: person:tobie, name: 'Tobie', opts: { enabled: true } }"

[[test.results]]
value = "{ email: 'tobie@surrealdb.com', name: 'Tobie' }"

*/

CREATE person:tobie SET name = 'Tobie', email = 'tobie@surrealdb.com', opts.enabled = true;

-- Select the whole record
person:tobie.*;

-- Select specific fields
person:tobie.{ name, email };
