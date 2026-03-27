/**[test]

[[test.results]]
value = "[]"

[[test.results]]
value = "[]"

[[test.results]]
value = "[]"

[[test.results]]
value = "[]"

*/

-- Don't return any result (the default)
DELETE user WHERE age < 18 RETURN NONE;

-- Return the changeset diff
DELETE user WHERE interests CONTAINS 'reading' RETURN DIFF;

-- Return the record before changes were applied
DELETE user WHERE interests CONTAINS 'reading' RETURN BEFORE;

-- Return the record after changes were applied
DELETE user WHERE interests CONTAINS 'reading' RETURN AFTER;
