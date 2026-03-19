/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ email: 'old_email@test.com', id: user:test }]"

[[test.results]]
value = "[{ email: 'new_email@test.com', id: user:test }]"

[[test.results]]
value = "[]"

[[test.results]]
value = "[{ action: 'email created', at: d'2025-10-07T05:57:35.399269Z', id: log:ehhri1t7006hxcsg8bb5, new_email: 'old_email@test.com', old_email: '', user: user:test }, { action: 'email updated', at: d'2025-10-07T05:57:39.687764Z', id: log:rg89w4a05zcgictbdk8d, new_email: 'new_email@test.com', old_email: 'old_email@test.com', user: user:test }, { action: 'email deleted', at: d'2025-10-07T05:58:15.120426Z', id: log:11dpjy6rrk23jse8ee8b, new_email: '', old_email: 'new_email@test.com', user: user:test }]"
skip-record-id-key = true
skip-datetime = true

*/

-- Create a new event whenever a user changes their email address
-- One-statement event
DEFINE EVENT OVERWRITE test ON TABLE user WHEN $before.email != $after.email THEN (
    CREATE log SET 
        user       = $value.id,
        // Turn events like "CREATE" into string "email created"
        action     = 'email' + ' ' + $event.lowercase() + 'd',
        // `email` field may be NONE, log as '' if so
        old_email  = $before.email ?? '',
        new_email  = $after.email  ?? '',
        at         = time::now()
);
UPSERT user:test SET email = 'old_email@test.com';
UPSERT user:test SET email = 'new_email@test.com';
DELETE user:test;
SELECT * FROM log ORDER BY at ASC;
