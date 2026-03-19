/**[test]

[[test.results]]
value = "[]"

[[test.results]]
value = "[[{ op: 'add', path: '/interests/1', value: 'reading' }]]"

[[test.results]]
value = "[{ id: person:tobie, interests: ['reading', 'reading'] }]"

[[test.results]]
value = "[{ id: person:tobie, interests: ['reading', 'reading', 'reading', 'reading'] }]"

[[test.results]]
value = "[{ interests: ['skiing', 'music'], name: NONE }]"

*/

-- Don't return any result
UPSERT person:tobie SET interests += 'reading' RETURN NONE;

-- Return the changeset diff
UPSERT person:tobie SET interests += 'reading' RETURN DIFF;

-- Return the record before changes were applied
UPSERT person:tobie SET interests += 'reading' RETURN BEFORE;

-- Return the record after changes were applied (the default)
UPSERT person:tobie SET interests += 'reading' RETURN AFTER;

-- Return a specific field only from the upserted records
UPSERT person:tobie SET interests = ['skiing', 'music'] RETURN name, interests;
