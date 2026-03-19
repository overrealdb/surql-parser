DEFINE INDEX idx ON indexed_reading COUNT;

FOR $_ IN 0..100000 {
    CREATE reading SET temperature = rand::int(0, 10);
};

FOR $_ IN 0..100000 {
    CREATE indexed_reading SET temperature = rand::int(0, 10);
};

-- Wait a moment before running these two
-- queries to ensure the index is built
SELECT count() FROM reading GROUP ALL;
SELECT count() FROM indexed_reading GROUP ALL;
