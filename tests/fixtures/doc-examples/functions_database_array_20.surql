/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "[[1, 2], [3, 4]]"

[[test.results]]
value = "[[1, 2, 3], [4]]"

*/

LET $array = [1, 2, 3, 4];
RETURN array::clump($array, 2);
RETURN array::clump($array, 3);
