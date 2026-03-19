/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ id: person:hw8ha95g4gbegaox34gw, name: 'Tobie Morgan Hitchcock' }]"
skip-record-id-key = true

[[test.results]]
value = "[{ id: person:q4htgstrmm5j4mvisnhp, name: 'Jaime Morgan Hitchcock' }]"
skip-record-id-key = true

*/

-- Define the parameter
LET $suffix = "Morgan Hitchcock";
-- Use the parameter
CREATE person SET name = "Tobie " + $suffix;
-- (Another way to do the same)
CREATE person SET name = string::join(" ", "Jaime", $suffix);
