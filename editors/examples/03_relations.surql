-- ═══════════════════════════════════════════════════════════
-- Relations & Graph traversal — SurrealDB 3+ rich syntax
-- ═══════════════════════════════════════════════════════════

-- Relation tables
DEFINE TABLE follows TYPE RELATION FROM user TO user ENFORCED SCHEMAFULL;
DEFINE FIELD since ON follows TYPE datetime DEFAULT time::now();
DEFINE FIELD strength ON follows TYPE string DEFAULT 'normal';

DEFINE TABLE wrote TYPE RELATION FROM user TO post ENFORCED SCHEMAFULL;
DEFINE FIELD published_at ON wrote TYPE datetime DEFAULT time::now();

DEFINE TABLE attends TYPE RELATION FROM user TO event ENFORCED SCHEMAFULL;

-- Create relations
RELATE user:alice->follows->user:bob SET since = time::now();
RELATE user:alice->follows->user:charlie SET strength = 'strong';
RELATE user:alice->wrote->post:hello SET published_at = d'2024-01-15T10:30:00Z';

-- Forward graph traversal
SELECT ->follows->user.name AS friends FROM user:alice;
SELECT ->follows->user->follows->user.name AS friends_of_friends FROM user:alice;
SELECT ->wrote->post.title AS posts FROM user:alice;

-- Reverse graph traversal
SELECT <-follows<-user.name AS followers FROM user:bob;
SELECT <-wrote<-user.name AS authors FROM post:hello;

-- Multi-hop traversal
SELECT ->follows->user->wrote->post.title AS friend_posts FROM user:alice;
SELECT ->follows->user->follows->user->follows->user AS three_hops FROM user:alice;

-- Mixed directions
SELECT
	id,
	->follows->user.name AS following,
	<-follows<-user.name AS followers,
	->wrote->post.title AS posts,
	->attends->event.{ title, date } AS events
FROM user:alice;

-- Filtered graph traversal
SELECT ->follows->(? WHERE strength = 'strong')->user.name AS close_friends
FROM user:alice;

-- Graph with aggregation
SELECT
	count(->follows->user) AS following_count,
	count(<-follows<-user) AS follower_count
FROM user;

-- Subquery with RELATE
LET $users = SELECT id FROM user WHERE age > 21;
RELATE $users->attends->event:conference;
