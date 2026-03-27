/**[test]

[[test.results]]
value = "0"

[[test.results]]
value = "NULL"

*/

RETURN array::logical_and([0, 1], [])

-- [ 0, NULL ]
