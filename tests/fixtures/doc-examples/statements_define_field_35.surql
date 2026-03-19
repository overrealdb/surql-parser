/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ coffee: { special_order: 'Venti Quadruple Ristretto Half-Decaf Soy Latte with 4 pumps of sugar-free vanilla syrup' }, id: order:good }]"

[[test.results]]
error = ""Couldn't coerce value for field `coffee` of `order:bad`: Expected `'regular' | 'large' | { special_order: string }` but found `'small'`""

*/

DEFINE FIELD coffee ON TABLE order TYPE "regular" | "large" | { special_order: string };

CREATE order:good SET coffee = { special_order: "Venti Quadruple Ristretto Half-Decaf Soy Latte with 4 pumps of sugar-free vanilla syrup" };
CREATE order:bad SET coffee = "small";
