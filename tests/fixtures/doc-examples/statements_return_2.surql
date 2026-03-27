/**[test]

[[test.results]]
value = "person:79e0p1et39n2dyongjzx"
skip-record-id-key = true

*/

BEGIN TRANSACTION;

-- We are executing quite a few queries here
LET $firstname = "John";
LET $lastname = "Doe";

LET $person = CREATE ONLY person CONTENT {
	firstname: $firstname,
	lastname: $lastname,
};

-- But because we end with a RETURN query, only the person's ID will be returned
-- The results of the other queries will be omitted.
RETURN $person.id;

-- One issue with this approach is that query errors are generic.
-- To get around that, use a block, which is executed as a transaction by itself.

COMMIT TRANSACTION;
