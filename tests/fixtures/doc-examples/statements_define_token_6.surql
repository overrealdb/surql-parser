-- Specify the namespace and database for the token
USE NS abcum DB app_vitalsense;

-- Necessary in order to define a scope token
DEFINE SCOPE users;

DEFINE TOKEN token_name ON SCOPE users TYPE RS256 VALUE "-----BEGIN PUBLIC KEY-----
MUO52Me9HEB4ZyU+7xmDpnixzA/CUE7kyUuE0b7t38oCh+sQouREqIjLwgHhFdhh3cQAwr6GH07D
ThioYrZL8xATJ3Youyj8C45QnZcGUif5PkpWXDi0HJSoMFekbW6Pr4xuqIqb2LGxGDVJcLZwJ2AS
Gtu2UAfPXbBD3ffiad393M22g1iHM80YaNi+xgswG7qtXE4lR/Lt4s0MeKKX7stdWI1VIsoB+y3i
r/OWUvJPjjDNbAsyy8tQmxydv+FUnLEP9TNT4AhN4DXcJ+XsDtW7OWt4EdSVDeKpGbIMvIrh1Pe+
Nilj8UHNyNDHa2AjK3seMo6CMvaIQJKj5o4xGFblFGwvvPD03SbuQLs1FdRjsZCeWLdYeQ3JDHE9
sFG7DCXlpMJcaYT1mf4XHJ0gPekNLQyewTY3Vxf7FgV3GCNjV20kcDFgJA2+iVW2wSrb+txD1ycE
kbi8jh0pedWwE40VQWaTh/8eAvX7IHWya/AEro25mq+m6vktNZLbvLphhp586kJK3Tdt3YjpkPre
M3nkFWOWurIyKbtIV9JemfwCgt89sNV45dTlnEDEZFFGnIgDnWgx3CUo4XmhICEQU8+tklw9jJYx
iCTjhbIDEBHySSSc/pQ4ftHQmhToTlQeOdEy4LYiaEIgl1X+hzRH1hBYvWlNKe4EY1nMCKcjgt0=
-----END PUBLIC KEY-----";

DEFINE TABLE user SCHEMAFULL
  -- Authorized users can select, update, delete and create user records
  PERMISSIONS FOR select, update, delete, create
  -- The current scope must be "users"
  WHERE $scope = "users"
  -- The email of the user being queried must match the email claim in the token
  -- Only matching records will be changed or returned
  AND email = $token.email
;

DEFINE INDEX email ON user FIELDS email UNIQUE;
DEFINE FIELD email ON user TYPE string ASSERT string::is_email($value);
DEFINE FIELD name ON user TYPE string;
DEFINE FIELD nickname ON user TYPE string;
DEFINE FIELD picture ON user TYPE string;
