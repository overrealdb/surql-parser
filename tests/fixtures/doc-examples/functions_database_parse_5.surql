/**[test]

[[test.results]]
value = "'/features'"

*/

RETURN parse::url::path("https://surrealdb.com:80/features?some=option#fragment");

-- '/features'
