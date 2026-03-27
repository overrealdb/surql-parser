-- No need for parentheses
DEFINE FIELD random_movie
    ON app_screen
    COMPUTED SELECT * FROM ONLY movie ORDER BY RAND() LIMIT 1;
