/**[test]

[[test.results]]
error = "'Incorrect arguments for function array::swap(). Argument 1 is out of range. Expected a number between -2 and 2'"

*/

RETURN array::swap([0, 1], 100, 1000000);
