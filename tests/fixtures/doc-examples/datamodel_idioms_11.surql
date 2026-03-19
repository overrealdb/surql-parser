/**[test]

[[test.results]]
value = "{ id: person:tobie, name: 'Tobie' }"

[[test.results]]
value = "{ id: person:tobie, name: 'Tobie' }"

[[test.results]]
value = "{ id: person:tobie, name: 'Tobie' }"

[[test.results]]
value = "{ id: person:tobie, name: 'Tobie' }"

[[test.results]]
value = "{ id: person:tobie, name: 'Tobie' }"

*/

CREATE ONLY person:tobie SET name = 'Tobie';
SELECT * FROM ONLY person:tobie.*;    -- This works
SELECT * FROM ONLY (person:tobie.*);  -- Equivalent to above
SELECT * FROM ONLY { id: person:tobie, name: 'tobie' }; -- Equivalent to above
(SELECT * FROM ONLY person:tobie).*; -- Equivalent to above
