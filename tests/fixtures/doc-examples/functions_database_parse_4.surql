/**[test]

[[test.results]]
value = "'surrealdb.com'"

[[test.results]]
value = "'127.0.0.1'"

*/

RETURN parse::url::host("https://surrealdb.com:80/features?some=option#fragment");
RETURN parse::url::host("http://127.0.0.1/index.html");
