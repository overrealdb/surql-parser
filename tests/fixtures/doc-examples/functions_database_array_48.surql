/**[test]

[[test.results]]
value = "[1, 2, 3, 4, 5]"

*/

array::insert([1, 2, 3, 4], 'and me', 0);
-- ['and me', 1, 2, 3, 4]

array::insert([1, 2, 3, 4], 'and me', -1);
-- [1, 2, 3, 'and me', 4]
