FOR $i IN 0..5 {
    CREATE user SET user_num = $i, name = "User number " + <string>user_num;
};
