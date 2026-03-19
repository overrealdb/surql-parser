-- Other clauses like `COMMENT` are fine
DEFINE INDEX idx ON users COUNT
    COMMENT "Users are expected to grow substantially so index the count"
    CONCURRENTLY;

-- But not `FIELD`
DEFINE INDEX idx2 ON person FIELD name;
