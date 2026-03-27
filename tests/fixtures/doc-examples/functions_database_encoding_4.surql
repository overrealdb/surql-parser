/**[test]

[[test.results]]
value = "'aGVsbG8'"

*/
RETURN encoding::base64::encode(<bytes>"hello");

-- 'aGVsbG8'
