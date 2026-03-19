/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE EVENT on_comment_created ON TABLE comment
    WHEN $event = "CREATE"
    THEN {
        -- Check if the post allows for adding comments.
        -- User record doesn't have access to the `post` table.
        IF $after.post.disable_comments {
            THROW "Can't create a comment - Comments are disabled for this post";
        };

        -- Set the `approved` field on the new comment - automatically approve
        -- comments made by the author of the post.
        -- For security reasons, record users don't have any permissions for the `approved` field.
        UPDATE $after.id SET
            approved = $after.post.author == $after.author;
    };
