-- Create record representing a user
CREATE user:1 CONTENT { name: "tobie" };
-- Define bearer access method to generate API keys for record users
DEFINE ACCESS api ON DATABASE TYPE BEARER FOR RECORD DURATION FOR GRANT 10d;
-- Generate bearer grant to be used by a specific automation belonging to the user
ACCESS api GRANT FOR RECORD user:1;
