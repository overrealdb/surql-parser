FOR $num IN 0..100 {
    CREATE user SET num = $num;
    sleep(1ms); -- Simulate a bit of time between user creation
};

SELECT * FROM user WHERE num IN 50..=51;
SELECT * FROM user START 50 LIMIT 2;
