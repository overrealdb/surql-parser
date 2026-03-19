/**[test]

[[test.results]]
value = "80"

*/

RETURN parse::url::port("https://surrealdb.com:80/features?some=option#fragment");

-- 80
