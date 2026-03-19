DEFINE FIELD OVERWRITE followers
    ON user
    VALUE <future> { (SELECT VALUE count FROM ONLY follower_count WHERE user = $parent.id LIMIT 1) ?? 0 };
