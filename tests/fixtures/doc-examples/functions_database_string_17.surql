/**[test]

[[test.results]]
value = "['this', 'is', 'a', 'list']"

*/

RETURN string::split('this, is, a, list', ', ');

-- ['this', 'is', 'a', 'list']
