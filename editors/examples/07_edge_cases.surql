-- ═══════════════════════════════════════════════════════════
-- Edge cases — syntax highlighting stress test
-- ═══════════════════════════════════════════════════════════

-- ─── String varieties ───
SELECT 'single quoted';
SELECT "double quoted";
SELECT 'escaped ''apostrophe''';
SELECT "escaped \"quotes\"";
SELECT 'unicode: 日本語 🚀 Ñoño';

-- ─── Number varieties ───
SELECT 42;
SELECT 3.14;
SELECT -1;
SELECT 1_000_000;
SELECT 1.5e10;
SELECT 0.001;

-- ─── Boolean & null ───
SELECT true, false, NONE, NULL;

-- ─── Duration literals ───
SELECT 1ns, 1µs, 1ms, 1s, 1m, 1h, 1d, 1w;
LET $timeout = 30s;
SLEEP 500ms;

-- ─── Datetime ───
SELECT d'2024-01-15T10:30:00Z';
SELECT d'2024-12-25';

-- ─── Record IDs ───
SELECT * FROM user:alice;
SELECT * FROM user:123;
SELECT * FROM user:⟨550e8400-e29b-41d4-a716-446655440000⟩;

-- ─── Backtick-escaped identifiers ───
DEFINE TABLE `my table` SCHEMALESS;
DEFINE FIELD `field name` ON `my table` TYPE string;
SELECT `field name` FROM `my table`;

-- ─── Operators ───
SELECT * FROM user WHERE age > 18;
SELECT * FROM user WHERE age >= 18;
SELECT * FROM user WHERE age < 65;
SELECT * FROM user WHERE age <= 65;
SELECT * FROM user WHERE age != 0;
SELECT * FROM user WHERE name IS NOT NONE;

-- ─── Contains operators ───
SELECT * FROM user WHERE tags CONTAINS 'admin';
SELECT * FROM user WHERE tags CONTAINSALL ['admin', 'dev'];
SELECT * FROM user WHERE tags CONTAINSANY ['admin', 'mod'];
SELECT * FROM user WHERE tags CONTAINSNONE ['banned'];

-- ─── Geo operators ───
-- SELECT * FROM restaurant WHERE location INSIDE $area;
-- SELECT * FROM city WHERE boundary INTERSECTS $route;

-- ─── Comments ───
-- Line comment
/* Block comment */
/*
  Multi-line
  block comment
*/
/* Nested /* comments */ work */
SELECT /* inline comment */ * FROM user;

-- ─── Complex nesting ───
SELECT * FROM (
	SELECT * FROM (
		SELECT name FROM user WHERE age > 18
	) WHERE name != 'test'
);

-- ─── Object literals ───
CREATE config SET
	database = {
		host: 'localhost',
		port: 8000,
		options: {
			strict: true,
			timeout: 30s
		}
	},
	features: ['auth', 'graphql', 'live'];

-- ─── Array operations ───
SELECT * FROM user WHERE 'admin' IN tags;
SELECT * FROM [1, 2, 3, 4, 5];
SELECT * FROM user WHERE age IN [18, 21, 30, 40, 50];

-- ─── Parameters in various positions ───
SELECT $fields FROM $table WHERE $condition ORDER BY $sort LIMIT $limit;
CREATE $table SET name = $name, data = $data;
RELATE $from->$edge->$to;

-- ─── Empty statements & semicolons ───
;;;
SELECT 1;
;
SELECT 2;
