CREATE |user:50000| SET name = id.id() RETURN NONE;
DEFINE INDEX unique_name ON TABLE user FIELDS name UNIQUE;
INFO FOR INDEX unique_name ON TABLE user;
