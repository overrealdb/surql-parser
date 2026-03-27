/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE EVENT user_created ON TABLE user WHEN $event = "CREATE" THEN (
    CREATE log SET table = "user", event = $event, created_at = time::now()
);
