-- Create an INDEX concurrently
DEFINE INDEX test ON user FIELDS email CONCURRENTLY;
INFO FOR INDEX test ON user;
INFO FOR INDEX test ON user;
