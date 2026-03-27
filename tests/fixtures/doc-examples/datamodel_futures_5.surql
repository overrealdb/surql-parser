DEFINE FIELD random_movie
    ON app_screen
    VALUE <future> { (SELECT * FROM ONLY movie ORDER BY RAND() LIMIT 1) };
