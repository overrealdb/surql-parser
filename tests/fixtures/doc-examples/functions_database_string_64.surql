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
value = "1f"

[[test.results]]
value = "0.8218673218673219f"

[[test.results]]
value = "0.6266233766233765f"

[[test.results]]
value = "0.4379509379509379f"

*/

LET $first     = "In a hole in the ground there lived a hobbit";
LET $same      = "In a hole in the ground there lived a hobbit";
LET $close     = "In a hole in the GROUND there lived a Hobbit";
LET $different = "A narrow passage holds four hidden treasures";
LET $short     = "Hi I'm Brian";

-- Returns 1
string::similarity::jaro($first, $same);
-- Returns 0.8218673218673219f
string::similarity::jaro($first, $close);
-- Returns 0.6266233766233765f
string::similarity::jaro($first, $different);
-- Returns 0.4379509379509379f
string::similarity::jaro($first, $short);
