/**[test]

[[test.results]]
value = "person:one"

[[test.results]]
value = "person:one"

[[test.results]]
value = "person:one"

[[test.results]]
value = "'person:one'"

[[test.results]]
value = "98dec"

[[test.results]]
value = "98dec"

[[test.results]]
value = "98"

*/

-- All return a record person:one
r"person:one";
<record>"person:one";
<record<person>>"person:one";
-- Returns a string 'person:one'
'person:one';

-- Both return a decimal 98dec
98dec;
<decimal>98;

-- Returns an int 98
98;
