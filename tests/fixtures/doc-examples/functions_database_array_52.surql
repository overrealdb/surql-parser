/**[test]

[[test.results]]
value = "'again and again and again'"

*/

RETURN array::join(["again", "again", "again"], " and ");

-- "again and again and again"
