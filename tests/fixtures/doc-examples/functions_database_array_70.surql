/**[test]

[[test.results]]
value = "false"

[[test.results]]
value = "true"

*/

RETURN array::matches([{id: r"ohno:0"}, {id: r"ohno:1"}], {id: r"ohno:1"});

-- [false, true]
