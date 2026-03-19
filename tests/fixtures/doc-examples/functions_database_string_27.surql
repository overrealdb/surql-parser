/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "0"

[[test.results]]
value = "7"

[[test.results]]
value = "34"

[[test.results]]
value = "38"

*/

LET $first     = "In a hole in the ground there lived a hobbit";
LET $same      = "In a hole in the ground there lived a hobbit";
LET $close     = "In a hole in the GROUND there lived a Hobbit";
LET $different = "A narrow passage holds four hidden treasures";
LET $short     = "Hi I'm Brian";

-- Returns 0
string::distance::osa($first, $same);
-- Returns 7
string::distance::osa($first, $close);
-- Returns 34
string::distance::osa($first, $different);
-- Returns 38
string::distance::osa($first, $short);
