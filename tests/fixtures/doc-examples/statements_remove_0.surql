REMOVE NAMESPACE surrealdb;

REMOVE DATABASE blog;

REMOVE USER writer ON NAMESPACE;

REMOVE USER writer ON DATABASE;

REMOVE ACCESS token ON NAMESPACE;

REMOVE ACCESS user ON DATABASE;

REMOVE EVENT new_post ON TABLE article;

-- Only works for Schemafull tables (i.e. tables with a schema)
REMOVE FIELD tags ON TABLE article;

REMOVE INDEX authors ON TABLE article;

REMOVE ANALYZER example_ascii;

REMOVE FUNCTION fn::update_author;

REMOVE PARAM $author;

REMOVE TABLE article;
