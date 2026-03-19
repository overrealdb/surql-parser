/**[test]

[[test.results]]
value = ""{ company: 'SURREALDB!!!!!', latest_version: '2.0' }""

*/

{ company: 'SurrealDB', latest_version: '2.0' }
    .chain(|$name| <string>$name)
    .replace('SurrealDB', 'SURREALDB!!!!!');
