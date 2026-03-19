/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ id: person:one }], [{ comments: [comment:rkvpjc0l9iruy21k89su], id: person:one }]"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ comments: [comment:rkvpjc0l9iruy21k89su, comment:tu5f72ouuydjvycg41st], id: person:one }]"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[]"

[[test.results]]
value = "[{ author: [person:one], id: comment:rkvpjc0l9iruy21k89su, text: 'Estonia is bigger than I expected!' }]"

*/

DEFINE FIELD comments ON person TYPE option<array<record<comment>>> REFERENCE ON DELETE UNSET;
DEFINE FIELD author ON comment COMPUTED <~person;

CREATE person:one;
UPDATE person:one SET comments += (CREATE ONLY comment SET text = "Estonia is bigger than I expected!").id;
-- Give this one a parameter name so it can be deleted later
LET $comment = CREATE ONLY comment SET text = "I don't get the joke here?";
UPDATE person:one SET comments += $comment.id;
-- Now delete it
DELETE $comment;
-- Only one comment shows up for person:one now
person:one.comments.*.*;
