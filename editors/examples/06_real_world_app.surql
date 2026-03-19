-- ═══════════════════════════════════════════════════════════
-- Real-world application schema — Social platform
-- ═══════════════════════════════════════════════════════════

-- ─── Namespace Setup ───

DEFINE NAMESPACE social;
DEFINE DATABASE main;
USE NS social DB main;

-- ─── Core Tables ───

DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD username ON user TYPE string ASSERT string::len($value) >= 3;
DEFINE FIELD email ON user TYPE string ASSERT string::is_email($value);
DEFINE FIELD display_name ON user TYPE option<string>;
DEFINE FIELD bio ON user TYPE option<string>;
DEFINE FIELD avatar_url ON user TYPE option<string>;
DEFINE FIELD joined_at ON user TYPE datetime DEFAULT time::now();
DEFINE FIELD verified ON user TYPE bool DEFAULT false;
DEFINE FIELD role ON user TYPE string DEFAULT 'user';

DEFINE INDEX user_username ON user FIELDS username UNIQUE;
DEFINE INDEX user_email ON user FIELDS email UNIQUE;

DEFINE TABLE post SCHEMAFULL;
DEFINE FIELD title ON post TYPE string ASSERT string::len($value) > 0;
DEFINE FIELD body ON post TYPE string;
DEFINE FIELD published ON post TYPE bool DEFAULT false;
DEFINE FIELD created_at ON post TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON post TYPE option<datetime>;
DEFINE FIELD view_count ON post TYPE int DEFAULT 0;
DEFINE FIELD tags ON post TYPE array<string> DEFAULT [];

DEFINE INDEX post_created ON post FIELDS created_at;
DEFINE INDEX post_tags ON post FIELDS tags;

DEFINE TABLE comment SCHEMAFULL;
DEFINE FIELD body ON comment TYPE string;
DEFINE FIELD created_at ON comment TYPE datetime DEFAULT time::now();

-- ─── Relation Tables ───

DEFINE TABLE follows TYPE RELATION FROM user TO user ENFORCED SCHEMAFULL;
DEFINE FIELD created_at ON follows TYPE datetime DEFAULT time::now();

DEFINE TABLE wrote TYPE RELATION FROM user TO post ENFORCED SCHEMAFULL;
DEFINE TABLE commented TYPE RELATION FROM user TO comment ENFORCED SCHEMAFULL;
DEFINE TABLE comment_on TYPE RELATION FROM comment TO post ENFORCED SCHEMAFULL;

DEFINE TABLE liked TYPE RELATION FROM user TO post ENFORCED SCHEMAFULL;
DEFINE FIELD created_at ON liked TYPE datetime DEFAULT time::now();

-- ─── Custom Functions ───

DEFINE FUNCTION fn::feed($user_id: record<user>, $limit: int) -> array {
	-- Get posts from followed users, ordered by recency
	LET $following = SELECT ->follows->user.id AS ids FROM $user_id;
	RETURN SELECT *,
		<-wrote<-user.username AS author,
		count(<-liked<-user) AS like_count,
		count(<-comment_on<-comment) AS comment_count
	FROM post
	WHERE <-wrote<-user.id IN $following.ids
		AND published = true
	ORDER BY created_at DESC
	LIMIT $limit;
};

DEFINE FUNCTION fn::user::profile($username: string) -> object {
	LET $user = SELECT * FROM user WHERE username = $username LIMIT 1;
	RETURN {
		user: $user,
		post_count: count(SELECT id FROM post WHERE <-wrote<-user.id = $user.id),
		follower_count: count(SELECT id FROM user WHERE ->follows->user.id = $user.id),
		following_count: count($user->follows->user)
	};
};

DEFINE FUNCTION fn::post::trending($hours: int, $limit: int) -> array {
	RETURN SELECT *,
		count(<-liked<-user) AS likes,
		count(<-comment_on<-comment) AS comments,
		(count(<-liked<-user) * 2 + count(<-comment_on<-comment)) AS score
	FROM post
	WHERE created_at > time::now() - type::duration($hours + 'h')
		AND published = true
	ORDER BY score DESC
	LIMIT $limit;
};

-- ─── Events for Audit ───

DEFINE EVENT user_signup ON user WHEN $event = 'CREATE' THEN {
	CREATE audit SET
		action = 'signup',
		user = $after.id,
		email = $after.email,
		timestamp = time::now();
};

DEFINE EVENT post_publish ON post WHEN $event = 'UPDATE' AND $before.published = false AND $after.published = true THEN {
	CREATE notification SET
		type = 'new_post',
		target = $after.id,
		message = string::concat($after.title, ' was published'),
		timestamp = time::now();
};

-- ─── Sample Queries ───

-- Create users
CREATE user:alice SET username = 'alice', email = 'alice@example.com', display_name = 'Alice', verified = true;
CREATE user:bob SET username = 'bob', email = 'bob@example.com', display_name = 'Bob';
CREATE user:charlie SET username = 'charlie', email = 'charlie@example.com';

-- Create relations
RELATE user:alice->follows->user:bob;
RELATE user:alice->follows->user:charlie;
RELATE user:bob->follows->user:alice;

-- Create content
LET $post = CREATE post SET title = 'Getting Started with SurrealDB', body = 'SurrealDB is amazing...', published = true, tags = ['surrealdb', 'database', 'tutorial'];
RELATE user:alice->wrote->$post;
RELATE user:bob->liked->$post;

-- Complex analytics query
SELECT
	username,
	count(->follows->user) AS following,
	count(<-follows<-user) AS followers,
	count(->wrote->post) AS posts,
	count(->liked->post) AS liked_posts,
	math::mean(->wrote->post.view_count) AS avg_views
FROM user
ORDER BY followers DESC;

-- Social graph analysis
SELECT
	username AS user,
	->follows->user.username AS following,
	->follows->user->follows->user.username AS fof,
	array::intersect(
		->follows->user.id,
		<-follows<-user.id
	) AS mutual_follows
FROM user:alice;
