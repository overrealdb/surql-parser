-- Core schema: users and posts

DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD name ON user TYPE string;
DEFINE FIELD email ON user TYPE string;
DEFINE FIELD age ON user TYPE int;
DEFINE INDEX email_idx ON user FIELDS email UNIQUE;

DEFINE TABLE post SCHEMAFULL;
DEFINE FIELD title ON post TYPE string;
DEFINE FIELD content ON post TYPE string;
DEFINE FIELD author ON post TYPE record<user>;
DEFINE FIELD created_at ON post TYPE datetime DEFAULT time::now();
