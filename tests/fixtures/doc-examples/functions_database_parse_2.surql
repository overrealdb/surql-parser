/**[test]

[[test.results]]
value = "'surrealdb.com"

[[test.results]]
value = 'NONE'

*/

RETURN parse::url::domain("https://surrealdb.com:80/features?some=option#fragment");
RETURN parse::url::domain("http://127.0.0.1/index.html");
