/**[test]

[[test.results]]
value = "true"

[[test.results]]
value = "true"

[[test.results]]
value = "true"

[[test.results]]
value = "true"

[[test.results]]
value = "true"

*/

RETURN [
    not(""),
    not(false),
    not([]),
    not({}),
    not(0)
];
