/**[test]

[[test.results]]
value = "'some=option'"

*/

RETURN parse::url::query("https://surrealdb.com:80/features?some=option#fragment");

-- 'some=option'
