DEFINE EVENT archive_person ON TABLE person WHEN $event = "DELETE" THEN {
    CREATE deleted_person SET
        data = $before,
        deleted_at = time::now()
};

CREATE |person:1..5|;
DELETE person:1;

-- Only two `person` records left
SELECT * FROM person;
-- But the data of `person:1` is still here
SELECT * FROM deleted_person;
