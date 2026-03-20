-- Custom SurrealQL functions

DEFINE FUNCTION fn::get_user($id: string) -> object {
	RETURN SELECT * FROM user WHERE id = $id;
};

DEFINE FUNCTION fn::create_post($title: string, $content: string, $author: record<user>) -> object {
	RETURN CREATE post SET title = $title, content = $content, author = $author;
};

DEFINE FUNCTION fn::user::list() -> array {
	RETURN SELECT * FROM user;
};

DEFINE FUNCTION fn::user::by_age($min_age: int, $max_age: int) -> array {
	RETURN SELECT * FROM user WHERE age >= $min_age AND age <= $max_age;
};

DEFINE
FUNCTION fn::heh::meh()-> array {
   RETURN [];
}
