/**[test]

[[test.results]]
value = "[{ id: person:jhn1rlhusqypyyyy0gjg, name: 'Billy', name_is_billy: true, name_is_long: false }]"
skip-record-id-key = true

*/


CREATE person SET 
    name = "Billy", 
    name_is_billy = name = "Billy",
    name_is_long = string::len(name) > 10;
