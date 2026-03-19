/**[test]

[[test.results]]
value = ""I_don't_like_whitespace""

*/

"I don't like whitespace"
  .split(" ")
  .fold("", |$one, $two, $index| IF $index = 0 { $one + $two } ELSE { $one + "_" + $two });
