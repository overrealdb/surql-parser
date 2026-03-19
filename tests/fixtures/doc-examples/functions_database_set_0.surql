/**
[test]

[[test.results]]
value = "{'one', 'three', 'two'}"
*/
RETURN set::add({"one", "two"}, "three");
-- {'one', 'three', 'two'}
