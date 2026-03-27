/**[test]

[[test.results]]
value = "[{ id: person:trgs6t98z3j5zlxsx1x5, name: 'John Doe' }, { id: person:xiysvpvqvglklrwy9tt5, name: 'John Doe' }, { id: person:qdpkqcb8gu5tt8jocuc8, name: 'Jane Doe' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: person:qdpkqcb8gu5tt8jocuc8, name: 'Jane Doe', people_with_same_name: [person:qdpkqcb8gu5tt8jocuc8] }, { id: person:trgs6t98z3j5zlxsx1x5, name: 'John Doe', people_with_same_name: [person:trgs6t98z3j5zlxsx1x5, person:xiysvpvqvglklrwy9tt5] }, { id: person:xiysvpvqvglklrwy9tt5, name: 'John Doe', people_with_same_name: [person:trgs6t98z3j5zlxsx1x5, person:xiysvpvqvglklrwy9tt5] }]"
skip-record-id-key = true

*/

INSERT INTO person (name) VALUES ("John Doe"), ("John Doe"), ("Jane Doe");
SELECT 
    *,
    (SELECT VALUE id FROM person WHERE $this.name = $parent.name) AS 
    people_with_same_name
    FROM person;
