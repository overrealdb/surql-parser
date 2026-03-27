/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

*/

REMOVE NAMESPACE IF EXISTS surrealdb;

REMOVE DATABASE IF EXISTS blog;

REMOVE USER IF EXISTS writer ON NAMESPACE;

REMOVE USER IF EXISTS writer ON DATABASE;

REMOVE ACCESS IF EXISTS token ON NAMESPACE;

REMOVE ACCESS IF EXISTS user ON DATABASE;

REMOVE EVENT IF EXISTS new_post ON TABLE article;

REMOVE FIELD IF EXISTS tags ON TABLE article;

REMOVE INDEX IF EXISTS authors ON TABLE article;

REMOVE ANALYZER IF EXISTS example_ascii;

REMOVE FUNCTION IF EXISTS fn::update_author;

REMOVE PARAM IF EXISTS $author;

REMOVE TABLE IF EXISTS article;
