/**[test]

[[test.results]]
value = "'Again and again and again'"

[[test.results]]
value = "'Again and again and again'"

*/

-- Traditional syntax
array::join(array::push(["Again", "again"], "again"), " and ");

-- Method chaining syntax
["Again", "again"].push("again").join(" and ");
