/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
match = "$error = "Couldn't coerce value for field `possessions` of `person:*`: Expected `none | array<record<any>>` but found `[book:one, house:one]`""
error = true

[[test.results]]
value = "[{ id: person:u6qd2t4ij2h45bkf2gk4, possessions: [any:one, any:two] }]"
skip-record-id-key = true

*/

DEFINE FIELD possessions ON TABLE person TYPE option<array<record<any>>>;

-- Won't work, 'book' and 'house' are not of table 'any'
CREATE person SET
    possessions = [ book:one, house:one ];

-- Actually expects this, which is probably
-- not what the DEFINE FIELD intended
CREATE person SET
    possessions = [ any:one, any:two ];
