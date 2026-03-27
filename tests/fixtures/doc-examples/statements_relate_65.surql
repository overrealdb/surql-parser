/**[test]

[[test.results]]
value = "[{ id: person:one }, { id: person:two }]"

[[test.results]]
value = "[{ id: likes:0ng6fka1m8iwp7y5lgul, in: person:one, out: person:two }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: likes:50kdcb8awc7jwsxzr7ra, in: person:two, out: person:one }]"
skip-record-id-key = true

[[test.results]]
error = "'Exceeded the idiom recursion limit of 256.'"

*/

CREATE person:one, person:two;
RELATE person:one->likes->person:two;
RELATE person:two->likes->person:one;
-- Open-ended range
person:one.{..}->likes->person;
