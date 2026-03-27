/**
[test]

[[test.results]]
value = "{1, 2, 3, 4, 5, 6, 7, 8, 'SurrealDB'}"
*/

-- Flattens everything except the array contained inside an inner set
({ {1, 2}, [3, 4], 'SurrealDB', {5, 6, [7, 8]} }).flatten();

-- Call twice: flattens everything including the nested array
({ {1, 2}, [3, 4], 'SurrealDB', {5, 6, [7, 8]} }).flatten().flatten();
