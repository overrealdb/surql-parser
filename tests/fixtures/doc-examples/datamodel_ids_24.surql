/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ id: user:01K6KTHCCJW7ZA6NWBZEFEVDC9, num: 50 }, { id: user:01K6KTHCCNRMN565T2H6JW70D6, num: 51 }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: user:01K6KTHCCJW7ZA6NWBZEFEVDC9, num: 50 }, { id: user:01K6KTHCCNRMN565T2H6JW70D6, num: 51 }]"
skip-record-id-key = true

*/

FOR $num IN 0..100 {
    CREATE user:ulid() SET num = $num;
    sleep(1ms);
};

SELECT * FROM user WHERE num IN 50..=51;
SELECT * FROM user START 50 LIMIT 2;
