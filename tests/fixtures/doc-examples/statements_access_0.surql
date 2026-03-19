-- Define system user for automation
DEFINE USER automation ON DATABASE PASSWORD 'secret' ROLES VIEWER;
-- Define bearer access method to generate API keys
DEFINE ACCESS api ON DATABASE TYPE BEARER FOR USER DURATION FOR GRANT 10d;
-- Generate bearer grant to be used by the automation
ACCESS api GRANT FOR USER automation;
