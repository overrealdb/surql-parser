/**[test]

[[test.results]]
error = "Incorrect arguments for function math::sum(). Argument 1 was the wrong type. Expected `number` but found `NONE` when coercing an element of `array<number>`"

*/

math::sum([0, NONE, 10dec, 10.7, NULL]);

-- Error: Incorrect arguments for function math::sum().
-- Argument 1 was the wrong type.
-- Expected `number` but found `NONE` when coercing an element of `array<number>`
