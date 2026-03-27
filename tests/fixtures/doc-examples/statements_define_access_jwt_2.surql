/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

*/

-- Specify the namespace and database for the token
USE NS abcum DB app_vitalsense;

-- Set the name of the token
DEFINE ACCESS token_name
  -- Use this token provider for database authentication
  ON DATABASE
  -- Specify the type of access being defined
  TYPE JWT
  -- Specify the URL where the JWKS object can be found
  URL "https://example.com/.well-known/jwks.json"
;
