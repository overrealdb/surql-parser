/**[test]

[[test.results]]
value = "['hello', 'hello']"

*/

RETURN array::repeat("hello", 2);

-- [ "hello", "hello" ]
