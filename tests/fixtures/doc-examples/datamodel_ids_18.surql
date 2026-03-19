FOR $i IN 0..5 {
    CREATE user SET user_num = $i, name = "User number " + <string>user_num;
};

LET $users = SELECT * FROM user;
FOR $user IN $users {
    LET $others = array::complement($users, [$user.id]);
    FOR $counterpart IN $others {
        IF rand::bool() {
            RELATE $user->likes->$counterpart;
        }
    }
};
