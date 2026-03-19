-- ═══════════════════════════════════════════════════════════
-- Advanced queries — subqueries, transactions, live, timeseries
-- ═══════════════════════════════════════════════════════════

-- Subqueries
SELECT *, (SELECT count() FROM ->wrote->post) AS post_count FROM user;
SELECT * FROM (SELECT name, age FROM user WHERE age > 18);
SELECT * FROM user WHERE id IN (SELECT author FROM post WHERE published = true);

-- Conditional expressions
SELECT *,
	IF age >= 18 THEN 'adult' ELSE 'minor' END AS category,
	IF active THEN 'yes' ELSE 'no' END AS status
FROM user;

-- Transactions
BEGIN TRANSACTION;
LET $u = CREATE user SET name = 'Eve', age = 28;
LET $p = CREATE post SET title = 'My First Post', body = 'Hello SurrealDB!';
RELATE $u->wrote->$p SET published_at = time::now();
COMMIT TRANSACTION;

-- Live queries
LIVE SELECT * FROM user WHERE active = true;
LIVE SELECT DIFF FROM post;

-- Timeseries with time grouping
SELECT
	time::group(created_at, 'hour') AS hour,
	count() AS events,
	math::mean(value) AS avg_value,
	math::max(value) AS peak
FROM metric
WHERE created_at > time::now() - 24h
GROUP BY hour
ORDER BY hour DESC;

-- Geospatial
SELECT *,
	geo::distance(location, { type: 'Point', coordinates: [40.7128, -74.0060] }) AS distance
FROM restaurant
WHERE geo::distance(location, { type: 'Point', coordinates: [40.7128, -74.0060] }) < 1000
ORDER BY distance
LIMIT 10;

-- Complex math
SELECT *,
	math::sqrt(math::pow(lat - $lat, 2) + math::pow(lon - $lon, 2)) AS distance
FROM location
ORDER BY distance
LIMIT $limit;

-- Record ID expressions
SELECT * FROM user:alice;
SELECT * FROM user:⟨550e8400-e29b-41d4-a716-446655440000⟩;

-- FOR loop
FOR $item IN [1, 2, 3, 4, 5] {
	CREATE counter SET value = $item;
};

-- THROW
IF $age < 0 {
	THROW 'Age cannot be negative';
};

-- SLEEP (testing/debugging)
SLEEP 100ms;

-- INFO
INFO FOR DB;
INFO FOR TABLE user;

-- SHOW changes
SHOW CHANGES FOR TABLE user SINCE d'2024-01-01T00:00:00Z';

-- RETURN
RETURN {
	users: (SELECT count() FROM user),
	posts: (SELECT count() FROM post),
	timestamp: time::now()
};

-- Explain
EXPLAIN SELECT * FROM user WHERE age > 18;
EXPLAIN FULL SELECT * FROM user WHERE email = 'alice@example.com';

-- Complex DEFINE with permissions
DEFINE FIELD email ON user TYPE string
	ASSERT string::is::email($value)
	PERMISSIONS
		FOR select WHERE true
		FOR create, update WHERE $auth.role = 'admin'
		FOR delete NONE;

-- Nested object access
SELECT
	settings.theme AS theme,
	settings.notifications.email AS email_notif,
	address.city AS city
FROM user;

-- Array indexing
SELECT tags[0] AS first_tag, tags[-1] AS last_tag FROM post;

-- Destructuring with aliases
SELECT
	name,
	{ age, email, active } AS profile,
	->follows->user.name AS friends
FROM user;
