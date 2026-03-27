/**[test]

[[test.results]]
value = "'fragment'"

*/

RETURN parse::url::fragment("https://surrealdb.com:80/features?some=option#fragment");

-- 'fragment'
