-- ═══════════════════════════════════════════════════════════
-- Basic SurrealQL — DML statements
-- ═══════════════════════════════════════════════════════════

-- Simple SELECT
SELECT * FROM user;
SELECT name, age FROM user WHERE age > 18 ORDER BY name ASC LIMIT 10;
SELECT country, count() AS total FROM user GROUP BY country;

-- CREATE with SET
CREATE user SET name = 'Alice', age = 30, active = true;
CREATE user CONTENT { name: 'Bob', age: 25, tags: ['admin', 'dev'] };

-- UPDATE
UPDATE user SET age = 31 WHERE name = 'Alice';
UPDATE user MERGE { settings: { theme: 'dark', lang: 'en' } } WHERE id = user:alice;

-- UPSERT
UPSERT user SET name = $name, email = $email, updated_at = time::now();

-- DELETE
DELETE user WHERE active = false;
DELETE user:alice;

-- INSERT
INSERT INTO user { name: 'Charlie', age: 28 };
INSERT INTO temperature [
	{ device: 'sensor-1', value: 22.5, ts: time::now() },
	{ device: 'sensor-2', value: 23.1, ts: time::now() },
];

-- LET bindings
LET $min_age = 18;
LET $results = SELECT * FROM user WHERE age > $min_age;

-- IF/ELSE
IF $results = NONE {
	RETURN 'No users found';
} ELSE {
	RETURN $results;
};
