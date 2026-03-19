/**[test]

[[test.results]]
value = "[true, false, true, false, true, false]"

*/

RETURN [
    !!"Has a value", !!"",             // true, false
    !!true, !!false,                   // true, false
    !!{ is_nice_database: true }, !!{} // true, false
    ];
