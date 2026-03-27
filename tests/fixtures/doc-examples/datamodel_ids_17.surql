FOR $user IN SELECT * FROM user {
    -- Use type::record to make a record ID
    -- from the user_num field
    CREATE type::record("user", $user.user_num);
    -- Then delete the old user
    DELETE $user;
};

SELECT * FROM user;
