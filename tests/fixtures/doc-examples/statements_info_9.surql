LET $cat = CREATE ONLY cat RETURN VALUE id;

LET $first_schema = {
    revision: rand::uuid(),
    schema: INFO FOR DB
};

$first_schema;

CREATE person SET feeds = [$cat];

LET $second_schema = {
    revision: rand::uuid(),
    schema: INFO FOR DB
};

$second_schema;

$first_schema.diff($second_schema);
