/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
error = "Expected `number` but found `'one'`"

[[test.results]]
error = "Couldn't coerce return value from function `fn::combine_any`: Expected `number` but found `'onetwo'`"

*/

// Arguments must be of type 'number'
DEFINE FUNCTION fn::combine($one: number, $two: number) -> number {
  $one + $two
};

// Accepts any value but expects the return type 'number'
DEFINE FUNCTION fn::combine_any($one: any, $two: any) -> number {
  $one + $two
};

fn::combine("one", "two");
fn::combine_any("one", "two");
