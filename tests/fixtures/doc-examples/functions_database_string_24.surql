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
value = "40"

[[test.results]]
value = "'Incorrect arguments for function string::distance::hamming(). Strings must be of equal length.'"

*/

LET $first     = "In a hole in the ground there lived a hobbit";
LET $same      = "In a hole in the ground there lived a hobbit";
LET $close     = "In a hole in the GROUND there lived a Hobbit";
LET $different = "A narrow passage holds four hidden treasures";
LET $short     = "Hi I'm Brian";

-- Returns 0
string::distance::hamming($first, $same);
-- Returns 7
string::distance::hamming($first, $close);
-- Returns 40
string::distance::hamming($first, $different);
-- Error: strings must be of equal length
string::distance::hamming($first, $short);
