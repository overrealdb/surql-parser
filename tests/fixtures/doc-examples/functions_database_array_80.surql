/**[test]

[[test.results]]
value = ""I_don't_like_whitespace""

*/

"I don't like whitespace"
  .split(" ")
  .reduce(|$one, $two| $one + "_" + $two);
