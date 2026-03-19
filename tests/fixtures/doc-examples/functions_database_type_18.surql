/**[test]

[[test.results]]
value = "1..2"

[[test.results]]
value = "1..10"

[[test.results]]
value = "'Expected a range but cannot convert [1, 9, 4] into a range'"

*/

RETURN type::range([1, 2]);
-- 1..2

RETURN type::range(1..10);
-- 1..10

RETURN type::range([1,9,4]);
-- 'Expected a range but cannot convert [1, 9, 4] into a range'
