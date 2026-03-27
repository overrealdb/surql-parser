/**[test]

[[test.results]]
value = "NONE"

*/

-- Redefine GraphQL configuration, overwriting existing configuration
DEFINE CONFIG OVERWRITE GRAPHQL TABLES INCLUDE user, post FUNCTIONS NONE;
