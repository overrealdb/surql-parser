/**[test]

[[test.results]]
value = ""_I_don't_like_whitespace""

*/

"I don't like whitespace"
  .split(" ")
  .fold("", |$one, $two| $one + "_" + $two);
