/**[test]

[[test.results]]
value = "'person'"

[[test.results]]
value = "'person'"

*/

-- Traditional syntax
record::table(array::max([r"person:aeon", r"person:landevin"]));

-- Method chaining syntax
[r"person:aeon", r"person:landevin"].max().table();
