/**[test]

[[test.results]]
value = "0.."

[[test.results]]
value = "0>.."

[[test.results]]
value = "..10"

[[test.results]]
value = "..=100"

[[test.results]]
value = ".."

*/

-- Anything from 0 and up
0..;
-- Anything from 1 and up
0>..;
-- Anything up to 99
..100;
-- Anything up to 100
..=100;
-- An infinite range
..;
