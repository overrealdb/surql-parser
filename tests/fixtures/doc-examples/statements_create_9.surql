-- Returns an array with a single record inside
CREATE person:tobie SET
    name = 'Tobie',
    company = 'SurrealDB',
    skills = ['Rust', 'Go', 'JavaScript'];

-- Returns just a single record
CREATE ONLY person:tobie SET
    name = 'Tobie',
    company = 'SurrealDB',
    skills = ['Rust', 'Go', 'JavaScript'];
