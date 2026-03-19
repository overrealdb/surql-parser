LET $id = 123;
LET $id = {
    IF $id {
        RETURN type::record('table', $id);
    };

    RETURN table:rand();
};

-- This still executes. The `RETURN` statement only broke until the block in the variable assignment.
$id;
