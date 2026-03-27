-- Specify the namespace and database for the token
USE NS abcum DB app_vitalsense;

-- Set the name of the token
DEFINE TOKEN token_name
  -- Use this token provider for database authorization
  ON DATABASE
  -- Specify the JWKS specification used to verify the token
  TYPE JWKS
  -- Specify the URL where the JWKS object can be found
  VALUE "https://example.com/.well-known/jwks.json"
;
