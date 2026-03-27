/**[test]

[[test.results]]
value = "['a', 'b']"

*/

RETURN object::keys({
  a: 1,
  b: true
});

-- [ 'a', 'b' ]
