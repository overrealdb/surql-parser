-- Define system user that access will be granted to
DEFINE USER automation ON DATABASE PASSWORD 'secret' ROLES VIEWER;
-- Define bearer access method to generate API keys for system users
DEFINE ACCESS api ON DATABASE TYPE BEARER FOR USER DURATION FOR GRANT 10d;
-- Generate bearer grant to be used by a specific automation
ACCESS api GRANT FOR USER automation;
