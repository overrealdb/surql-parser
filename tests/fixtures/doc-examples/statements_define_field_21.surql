DEFINE FIELD accessed_at ON TABLE user COMPUTED time::now();

CREATE user:one;
SELECT * FROM ONLY user:one;
-- Sleep for one second
SLEEP 1s;
-- `accessed_at` is a different value now
SELECT * FROM ONLY user:one;
