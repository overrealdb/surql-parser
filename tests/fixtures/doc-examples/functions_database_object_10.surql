/**[test]

[[test.results]]
value = "[1, true]"

*/

RETURN object::values({
  a: 1,
  b: true
});

-- [1, true]
