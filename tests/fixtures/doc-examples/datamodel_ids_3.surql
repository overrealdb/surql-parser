/**[test]

[[test.results]]
value = "[{ id: company:surrealdb, name: 'SurrealDB' }]"

[[test.results]]
value = "[{ id: user_version_2025:aajz3qh1nk0b27tztsa7, name: 'Alucard' }]"
skip-record-id-key = true

*/

CREATE company:surrealdb SET name = 'SurrealDB';
CREATE user_version_2025 SET name = 'Alucard';
