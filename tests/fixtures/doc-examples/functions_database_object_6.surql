/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
match = "$error = Found {  } for field `metadata`, with record `house:.*`, but field must conform to: !$value.is_empty()'"
error = true

[[test.results]]
value = "[{ id: house:ur51z2flendqdp3i130o, metadata: { floors: 5 } }]"
skip-record-id-key = true

*/

DEFINE FIELD metadata
  ON house
  TYPE object
  ASSERT !$value.is_empty();
CREATE house SET metadata = {};
CREATE house SET metadata = { floors: 5 };
