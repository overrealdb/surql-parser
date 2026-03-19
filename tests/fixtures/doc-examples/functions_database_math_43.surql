/**[test]

[[test.results]]
value = "901"

[[test.results]]
value = "2"

*/

RETURN math::mode([ 1, 40, 60, 10, 2, 901 ]);
-- 901

RETURN math::mode([ 1, 40, 60, 10, 2, 901, 2 ]);
-- 2
