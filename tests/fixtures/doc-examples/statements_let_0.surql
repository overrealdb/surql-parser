/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ id: person:qt3itwoql7oodlg3n077, name: 'tobie' }]"
skip-record-id-key = true

*/

-- Define the parameter
LET $name = "tobie";
-- Use the parameter
CREATE person SET name = $name;
