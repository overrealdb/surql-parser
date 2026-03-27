/**[test]

[[test.results]]
value = "[{ emails: ['john@example.com', 'doe@example.com'], id: user:jhjsg6c9ta32gdut36b8, name: 'John Doe' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ emails: 'john@example.com', id: user:jhjsg6c9ta32gdut36b8, name: 'John Doe' }, { emails: 'doe@example.com', id: user:jhjsg6c9ta32gdut36b8, name: 'John Doe' }]"
skip-record-id-key = true

*/
CREATE user SET
    name = "John Doe",
    emails = ["john@example.com", "doe@example.com"];

-- Split the results by each value in the emails array
SELECT * FROM user SPLIT emails;
