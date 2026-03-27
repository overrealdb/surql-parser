-- ═══════════════════════════════════════════════════════════
-- Edge cases — syntax highlighting stress test
-- ═══════════════════════════════════════════════════════════

-- ─── String varieties ───
RETURN 'single quoted';
RETURN "double quoted";
RETURN "escaped 'apostrophe'";
RETURN 'escaped "quotes"';
RETURN 'unicode: 日本語 🚀 Ñoño';

-- ─── Number varieties ───
RETURN 42;
RETURN 3.14;
RETURN -1;
RETURN 1_000_000;
RETURN 1.5e10;
RETURN 0.001;

-- ─── Boolean & null ───
RETURN [true, false, NONE, NULL];

-- ─── Duration literals ───
RETURN [1ns, 1µs, 1ms, 1s, 1m, 1h, 1d, 1w];
LET $timeout = 30s;
SLEEP 500ms;

-- ─── Datetime ───
RETURN d'2024-01-15T10:30:00Z';
RETURN d'2024-12-25';

-- ─── Record IDs ───
SELECT * FROM agent:overseer;
SELECT * FROM project:surql_parser;
SELECT * FROM migration:v001;

-- ─── Backtick-escaped identifiers ───
DEFINE TABLE `my table` SCHEMALESS;
DEFINE FIELD `field name` ON `my table` TYPE string;
SELECT `field name` FROM `my table`;

-- ─── Operators ───
SELECT * FROM agent WHERE active = true;
SELECT * FROM migration WHERE version > 1;
SELECT * FROM migration WHERE version >= 2;
SELECT * FROM migration WHERE version < 10;
SELECT * FROM spec WHERE status != 'draft';
SELECT * FROM agent WHERE model IS NOT NONE;

-- ─── Contains operators ───
SELECT * FROM schema_diff WHERE missing_tables CONTAINS 'agent';
SELECT * FROM schema_diff WHERE missing_tables CONTAINSALL ['agent', 'project'];
SELECT * FROM schema_diff WHERE extra_tables CONTAINSANY ['audit', 'temp'];
SELECT * FROM schema_diff WHERE extra_functions CONTAINSNONE ['fn::test'];

-- ─── Comments ───
-- Line comment
/* Block comment */
/*
  Multi-line
  block comment
*/
SELECT * FROM project;

-- ─── Complex nesting ───
SELECT * FROM (
	SELECT * FROM (
		SELECT name FROM agent WHERE active = true
	) WHERE name != 'test'
);

-- ─── Object literals ───
CREATE project:config SET
	settings = {
		endpoint: 'ws://localhost:8000',
		options: {
			verify: true,
			timeout: 30s
		}
	},
	tags = ['production', 'monitored'];

-- ─── Array operations ───
SELECT * FROM agent WHERE 'deployer' IN role;
SELECT * FROM [1, 2, 3, 4, 5];
SELECT * FROM migration WHERE version IN [1, 2, 3];

-- ─── Parameters in various positions ───
SELECT $fields FROM $table WHERE $condition LIMIT $limit;
CREATE $table SET name = $name, data = $data;
RELATE $from->$edge->$to;

-- ─── Empty statements & semicolons ───
;;;
RETURN 1;
;
RETURN 2;
