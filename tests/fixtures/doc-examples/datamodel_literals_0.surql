/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
error = ""Tried to set `$nine`, but couldn't coerce value: Expected `9` but found `10`""

*/

LET $nine: 9 = 9;
LET $nine: 9 = 10;
