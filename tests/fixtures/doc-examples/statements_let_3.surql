/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "'Before!'"

[[test.results]]
value = "[{ before: { id: person:1 } }, { before: { id: person:2 } }, { before: { id: person:3 } }, { before: { id: person:qt3itwoql7oodlg3n077, name: 'tobie' } }]"
skip-record-id-key = true

[[test.results]]
value = "'Before!'"

*/

LET $before = "Before!";

-- Returns ["Before!"];
RETURN $before;

-- Returns the `person` records before deletion
DELETE person RETURN $before;

-- Returns "Before!" again
RETURN $before;
