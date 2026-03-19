/**[test]

[[test.results]]
value = "5"

*/

RETURN count([ 1, 2, 3, null, 0, false, (15 > 10), rand::uuid() ]);

5
