/**[test]

[[test.results]]
value = "2"

[[test.results]]
value = "2"

*/

-- Traditional syntax
array::max(object::values(object::from_entries([["a", 1], ["b", 2]])));

-- Method chaining syntax
object::from_entries([["a", 1], ["b", 2]]).values().max();
