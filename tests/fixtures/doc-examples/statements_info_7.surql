CREATE |user:50000| SET name = id.id() RETURN NONE;
DEFINE INDEX unique_name ON TABLE user FIELDS name UNIQUE CONCURRENTLY;
INFO FOR INDEX unique_name ON user;
SLEEP 50ms;
INFO FOR INDEX unique_name ON user;
SLEEP 50ms;
INFO FOR INDEX unique_name ON user;
SLEEP 50ms;
INFO FOR INDEX unique_name ON user;
