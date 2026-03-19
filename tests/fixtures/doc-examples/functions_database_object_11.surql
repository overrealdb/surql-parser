/**[test]

[[test.results]]
value = "[1, true]"

[[test.results]]
value = "[1, true]"

*/

-- Traditional syntax
object::values({
  a: 1,
  b: true
});

-- Method chaining syntax
{
  a: 1,
  b: true
}.values();
