/**[test]

[[test.results]]
value = "[{ emails: ['me@me.com', 'longer_email@other_service.com'], id: user:czs7vds6shgb6m2hbwgj, name: 'Name' }]"

[[test.results]]
value = "[{ emails: 'me@me.com', id: user:czs7vds6shgb6m2hbwgj, name: 'Name' }, { emails: 'longer_email@other_service.com', id: user:czs7vds6shgb6m2hbwgj, name: 'Name' }]"

*/

CREATE user SET
    name = "Name",
    emails = ["me@me.com", "longer_email@other_service.com"];

-- Split the results by each value in an array
SELECT * FROM user SPLIT emails;
