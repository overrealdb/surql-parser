/**[test]

[[test.results]]
error = ""Tried to set `$nine`, but couldn't coerce value: Expected `9 | '9' | 'nine'` but found `'Nein'`""

*/

LET $nine: 9 | "9" | "nine" = "Nein";
