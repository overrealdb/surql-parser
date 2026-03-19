/**[test]

[[test.results]]
value = "[NULL, 0, 1, 1, 2, 3, 3, 4, 'something']"

*/

RETURN array::sort([1, 2, 1, null, "something", 3, 3, 4, 0], "asc");

-- [ null, 0, 1, 1, 2, 3, 3, 4, "something" ]
