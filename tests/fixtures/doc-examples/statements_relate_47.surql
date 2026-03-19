/**[test]

[[test.results]]
value = "[{ id: cat:one }, { id: cat:two }, { id: cat:three }]"

[[test.results]]
value = "[{ id: friends_with:8tuumzamsov3pro9tz3j, in: cat:one, out: cat:two }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: friends_with:ro94pfxfpff35x94kgmy, in: cat:two, out: cat:three }]"
skip-record-id-key = true

[[test.results]]
value = "[{ "->friends_with": { "->cat": { "->friends_with": { "->cat": [cat:three] } } } }]"

[[test.results]]
value = "[{ friends_of_friends: [cat:three] }]"

*/

CREATE cat:one, cat:two, cat:three;

RELATE cat:one->friends_with->cat:two;
RELATE cat:two->friends_with->cat:three;

SELECT ->friends_with->cat->friends_with->cat FROM cat:one;
-- create an alias for the result using the `AS` clause.
SELECT ->friends_with->cat->friends_with->cat AS friends_of_friends FROM cat:one;
