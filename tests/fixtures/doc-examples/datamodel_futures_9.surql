CREATE |person:10| SET name = "Person " + <string>id.id() RETURN NONE;

DEFINE FIELD random_friend
    ON person
    VALUE <future> { (SELECT VALUE name FROM ONLY person ORDER BY RAND() LIMIT 1) };

CREATE person;
