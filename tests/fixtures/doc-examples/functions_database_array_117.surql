/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = ""[[1, 2], [2, 3], [3, 4]]

[[test.results]]
value = "[]"

*/

LET $array = [1, 2, 3, 4];
RETURN array::windows($array, 2);
RETURN array::windows($array, 5);
