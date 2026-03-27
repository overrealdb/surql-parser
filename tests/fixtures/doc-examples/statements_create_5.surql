/**[test]

[[test.results]]
value = "[{ created_at: d'2025-10-08T02:18:52.928229Z', id: townsperson:0jkw66fganx0eqdkimgd, name: 'Just a townsperson' }, { created_at: d'2025-10-08T02:18:52.932072Z', id: cat:yw43e5ycrdq8mpzx7zq3, name: 'Just a cat' }, { created_at: d'2025-10-08T02:18:52.934887Z', id: dog:z4yq39x2hj79jl43v57x, name: 'Just a dog' }]"
skip-datetime = true 
skip-record-id-key = true

*/

-- Note: record::tb(id) returns just the table name portion of a record ID
CREATE townsperson, cat, dog SET
    created_at = time::now(),
    name = "Just a " + record::tb(id);
