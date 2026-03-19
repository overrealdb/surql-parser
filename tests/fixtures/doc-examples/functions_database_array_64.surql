/**[test]
[[test.results]]
error = ""Couldn't coerce return value from function `ANONYMOUS`: Expected `int` but found `2.1f`""

*/

[1, 2, 3].map(|$num: int| -> int { $num + 1.1 });
