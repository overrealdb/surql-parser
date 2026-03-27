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
    not(not("I have value")),
    not(not(true)),
    not(not(["value!"])),
    not(not({i_have: "value"})),
    not(not(100))
];
