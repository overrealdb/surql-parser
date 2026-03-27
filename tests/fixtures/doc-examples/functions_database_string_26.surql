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
value = "0.8409090909090909f"

[[test.results]]
value = "0.20454545454545459f"

[[test.results]]
value = "0.13636363636363635f"

*/

LET $first     = "In a hole in the ground there lived a hobbit";
LET $same      = "In a hole in the ground there lived a hobbit";
LET $close     = "In a hole in the GROUND there lived a Hobbit";
LET $different = "A narrow passage holds four hidden treasures";
LET $short     = "Hi I'm Brian";

-- Returns 1
string::distance::normalized_levenshtein($first, $same);
-- Returns 0.8409090909090909f
string::distance::normalized_levenshtein($first, $close);
-- Returns 0.20454545454545459f
string::distance::normalized_levenshtein($first, $different);
-- Returns 0.13636363636363635f
string::distance::normalized_levenshtein($first, $short);
