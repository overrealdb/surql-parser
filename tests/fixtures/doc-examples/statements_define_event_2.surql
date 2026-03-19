/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

*/

-- CREATE event is triggered when a new record is inserted into the table.
-- Here we are updating the status of the post to PUBLISHED
-- when a new record is inserted into the publish_post table.
DEFINE EVENT publish_post ON TABLE publish_post
    WHEN $event = "CREATE"
    THEN (
        UPDATE post SET status = "PUBLISHED" WHERE id = $after.post_id
    );

-- UPDATE event
-- Here we are creating a notification when a user is updated.
DEFINE EVENT user_updated ON TABLE user
    WHEN $event = "UPDATE"
    THEN (
        CREATE notification SET message = "User updated", user_id = $after.id, created_at = time::now()
    );

-- DELETE event is triggered when a record is deleted from the table.
-- Here we are creating a notification when a user is deleted.
DEFINE EVENT user_deleted ON TABLE user
    WHEN $event = "DELETE"
    THEN (
        CREATE notification SET message = "User deleted", user_id = $before.id, created_at = time::now()
    );

-- You can combine multiple events based on your use cases.
-- Here we are creating a log when a user is created, updated or deleted.
DEFINE EVENT user_event ON TABLE user
    WHEN $event = "CREATE" OR $event = "UPDATE" OR $event = "DELETE"
    THEN (
        CREATE log SET
            table = "user",
            event = $event,
            happened_at = time::now()
    );
