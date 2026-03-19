-- ═══════════════════════════════════════════════════════════
-- Functions — custom & built-in
-- ═══════════════════════════════════════════════════════════

-- Custom functions
DEFINE FUNCTION fn::greet($name: string) -> string {
	RETURN 'Hello, ' + $name + '!';
};

DEFINE FUNCTION fn::add($a: int, $b: int) -> int {
	RETURN $a + $b;
};

DEFINE FUNCTION fn::user::create($name: string, $email: string) -> object {
	LET $user = CREATE user SET name = $name, email = $email;
	RETURN $user;
};

DEFINE FUNCTION fn::user::by_age($min: int, $max: int) -> array {
	RETURN SELECT * FROM user WHERE age >= $min AND age <= $max;
};

DEFINE FUNCTION fn::post::publish($title: string, $body: string, $author: record<user>) -> object {
	LET $post = CREATE post SET title = $title, body = $body;
	RELATE $author->wrote->$post SET published_at = time::now();
	RETURN $post;
};

-- Calling custom functions
LET $greeting = fn::greet('World');
LET $sum = fn::add(40, 2);
LET $new_user = fn::user::create('Dave', 'dave@example.com');

-- Built-in string functions
SELECT
	string::len(name) AS name_length,
	string::uppercase(name) AS upper_name,
	string::lowercase(email) AS lower_email,
	string::trim(bio) AS clean_bio,
	string::slug(title) AS slug,
	string::concat(name, ' (', email, ')') AS display
FROM user;

-- Built-in math functions
SELECT
	math::sqrt(x * x + y * y) AS distance,
	math::round(price * 1.1, 2) AS with_tax,
	math::mean(scores) AS average,
	math::sum(quantities) AS total
FROM measurements;

-- Built-in time functions
SELECT
	time::now() AS current,
	time::floor(created_at, 1h) AS hour_bucket,
	time::group(created_at, 'day') AS day,
	time::year(created_at) AS year
FROM events;

-- Built-in array functions
SELECT
	array::len(tags) AS tag_count,
	array::distinct(tags) AS unique_tags,
	array::flatten(nested_array) AS flat,
	array::union(tags, ['new']) AS merged
FROM post;

-- Type functions
SELECT
	type::thing('user', id) AS record,
	type::is::string(name) AS is_str,
	type::is::number(age) AS is_num
FROM data;

-- Crypto
SELECT crypto::argon2::generate(password) AS hashed FROM input;

-- Random
SELECT
	rand::uuid::v4() AS id,
	rand::string(16) AS token,
	rand::int(1, 100) AS random_number;

-- Count and aggregation
SELECT
	count() AS total,
	count(active = true) AS active_count,
	math::min(age) AS youngest,
	math::max(age) AS oldest
FROM user;
