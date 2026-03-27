-- Define GraphQL configuration to include all tables automatically
DEFINE CONFIG GRAPHQL TABLES AUTO;

-- Define a table and insert data
DEFINE TABLE foo SCHEMAFUL;
DEFINE FIELD val ON foo TYPE int;
CREATE foo:1 SET val = 42;
CREATE foo:2 SET val = 43;
