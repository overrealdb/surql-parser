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
value = "0.8931203931203932f"

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

-- Returns 0
string::similarity::jaro_winkler($first, $same);
-- Returns 0.8931203931203932f
string::similarity::jaro_winkler($first, $close);
-- Returns 0.6266233766233765f
string::similarity::jaro_winkler($first, $different);
-- Returns 0.4379509379509379f
string::similarity::jaro_winkler($first, $short);
