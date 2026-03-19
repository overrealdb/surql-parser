/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ id: house:one, using: [] }]"

[[test.results]]
value = "[{ connected_to: [house:one], id: utility:gas }, { connected_to: [house:one], id: utility:water }]"

*/

DEFINE FIELD connected_to ON utility TYPE option<array<record<house>>> REFERENCE ON DELETE REJECT;
DEFINE FIELD using ON house COMPUTED <~utility;

CREATE house:one;
CREATE utility:gas, utility:water SET connected_to = [house:one];
