/**[test]

[[test.results]]
value = "[{ age: 46, id: person:ncnnp25vhiqddlfvz5ve, username: 'john-smith' }]"
skip-record-id-key = true

*/

-- Return the record after creation
CREATE person SET age = 46, username = "john-smith" RETURN AFTER;
