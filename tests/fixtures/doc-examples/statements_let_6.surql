/**[test]

[[test.results]]
value = "[{ id: some:record, vals: ['true', 'false', 'true'] }]"

[[test.results]]
value = "0"

*/

CREATE some:record SET vals = ["true", "false", "true"];
some:record.vals.filter(|$val| $val = true).len();
