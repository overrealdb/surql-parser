/**[test]

[[test.results]]
value = "'surrealdb.com'"

*/

RETURN parse::email::host("info@surrealdb.com");

-- 'surrealdb.com'
