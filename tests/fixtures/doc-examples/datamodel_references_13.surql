-- Non-optional field that attempts an UNSET when referencing 'person' is deleted
DEFINE FIELD author ON comment TYPE record<person> REFERENCE ON DELETE UNSET;
LET $person = CREATE ONLY person;
CREATE comment SET text = "Cats are so much better at climbing UP a tree than down! Lol", author = $person.id;
DELETE person;

-- Optional field which rejects the deletion of a referencing 'person'
DEFINE FIELD author ON comment TYPE option<record<person>> REFERENCE ON DELETE REJECT;
LET $person = CREATE ONLY person;
CREATE comment SET text = "Cats are so much better at climbing UP a tree than down! Lol", author = $person.id;
DELETE person;
