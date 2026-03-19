-- Create records representing users
CREATE user:1 CONTENT { name: "tobie" };
CREATE user:2 CONTENT { name: "jaime" };
-- Define bearer access method to generate API keys
DEFINE ACCESS api ON DATABASE TYPE BEARER FOR RECORD DURATION FOR GRANT 10d;
-- Generate bearer grants to be used by the users
ACCESS api GRANT FOR RECORD user:1;
ACCESS api GRANT FOR RECORD user:2;
