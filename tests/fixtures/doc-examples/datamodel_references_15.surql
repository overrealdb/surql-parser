/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ comments: [comment:7g857mbox5rdqvjuezxf], id: person:one }]"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ comments: [comment:7g857mbox5rdqvjuezxf, comment:o7qgd26uruvf3hracvrh], id: person:one }]"

[[test.results]]
value = "[]"

[[test.results]]
value = "[{ comments: [comment:7g857mbox5rdqvjuezxf], deleted_comments: [comment:o7qgd26uruvf3hracvrh], id: person:one }]"

*/

DEFINE FIELD comments ON person TYPE option<array<record<comment>>> REFERENCE ON DELETE THEN {
    UPDATE $this SET
        deleted_comments += $reference,
        comments -= $reference;
};
DEFINE FIELD author ON comment COMPUTED <~person;

CREATE person:one SET comments += (CREATE ONLY comment SET text = "Estonia is bigger than I expected!").id;
LET $comment = CREATE ONLY comment SET text = "I don't get the joke here?";
UPDATE person:one SET comments += $comment.id;
DELETE $comment;
SELECT * FROM person:one;
