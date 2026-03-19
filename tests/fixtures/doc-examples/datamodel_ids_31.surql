/**[test]

[[test.results]]
value = "[{ id: person_id:counter, num: 1 }]"

[[test.results]]
value = "[{ id: person:1 }]"

[[test.results]]
error = "'The query was not executed due to a failed transaction'"

[[test.results]]
error = ""Expected `datetime` but found a `'2025_01+01'`""

[[test.results]]
value = "1"

*/

BEGIN TRANSACTION;
UPSERT person_id:counter SET num += 1;
-- Creates a person:1
CREATE type::record("person", person_id:counter.num);
COMMIT TRANSACTION;

BEGIN TRANSACTION;
-- Latest ID is now 2
UPSERT person_id:counter SET num += 1;
-- Whoops, invalid datetime format
-- Transaction fails and all changes are rolled back
CREATE type::record("person", person_id:counter.num) SET created_at = <datetime>'2025_01+01';
COMMIT TRANSACTION;

-- Latest ID is still 1
RETURN person_id:counter.num;
