/**[test]

[[test.results]]
value = "[{ id: person:one }, { id: person:two }, { id: person:three }]"

[[test.results]]
value = "[{ id: likes:qv1sjo66j48irzu1hsik, in: person:one, out: person:two }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: likes:w4n1on9bim3w1d7r3tg3, in: person:two, out: person:three }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: likes:1jopms33qaetjmtv3ygs, in: person:three, out: person:one }]"
skip-record-id-key = true

[[test.results]]
value = "[]"

[[test.results]]
value = "[{ id: likes:1jopms33qaetjmtv3ygs, in: person:three, out: person:one }]"
skip-record-id-key = true

*/

-- Create three people
CREATE person:one, person:two, person:three;

-- And a love triangle involving them all
RELATE person:one  ->likes->person:two;
RELATE person:two  ->likes->person:three;
RELATE person:three->likes->person:one;

-- Person two moves to Venus permanently, so delete
DELETE person:two;

-- Only one `likes` relationship is left
SELECT * FROM likes;
