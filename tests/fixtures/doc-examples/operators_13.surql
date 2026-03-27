/**[test]

[[test.results]]
value = "[false]"

*/

SELECT * FROM { this: "object" } = { another: "object" };
-- false
