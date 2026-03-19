DEFINE FIELD string_test ON test TYPE string;
DEFINE FIELD int_test ON test TYPE int;
DEFINE FIELD datetime_test ON test TYPE datetime;

CREATE |test:10000| SET 
    string_test = rand::string(10),
    int_test = rand::int(),
    datetime_test = rand::time()
RETURN NONE;

FOR $field IN (INFO FOR TABLE test).fields.keys() {
    LET $now = time::now();
    REMOVE FIELD $field ON test;
    LET $elapsed = time::now() - $now;
    CREATE log SET results = { field_name: $field, removed_in: $elapsed }
};
