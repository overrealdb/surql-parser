/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ friends: [person:one, person:two], id: person:663uogu8gnw31irybeer, possessions: [book:one, house:one] }]"
skip-record-id-key = true

*/

DEFINE FIELD possessions ON TABLE person TYPE option<array<record>>;
DEFINE FIELD friends ON TABLE person TYPE option<array<record<person>>>;

CREATE person SET
    possessions = [ book:one, house:one],
    friends = [ person:one, person:two ];
