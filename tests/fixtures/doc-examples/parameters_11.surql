DEFINE TABLE user SCHEMAFULL
    PERMISSIONS
        FOR select, update, delete WHERE id = $auth.id;
