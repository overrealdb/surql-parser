/**[test]

[[test.results]]
value = "true"

[[test.results]]
value = "true"

*/

[
  string::matches("grey", "gr(a|e)y"), 
  string::matches("gray", "gr(a|e)y")
];

-- [true, true]
