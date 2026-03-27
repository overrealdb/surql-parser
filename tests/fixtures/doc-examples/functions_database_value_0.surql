/**[test]

[[test.results]]
value = "'SurrealDB 3.0'"

*/

'SurrealDB'.chain(|$n| $n + ' 3.0');
-- 'SurrealDB 3.0'

'SurrealDB'.chain(|$n| "Something else");
