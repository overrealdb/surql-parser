CREATE dog, |cat:2|, |townsperson:1..3| SET
    created_at = time::now(),
    name = "Just a " + record::tb(id);
