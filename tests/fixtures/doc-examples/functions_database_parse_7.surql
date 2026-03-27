/**[test]

[[test.results]]
value = "'https'"

*/

RETURN parse::url::scheme("https://surrealdb.com:80/features?some=option#fragment");

-- 'https'
