/**[test]

[[test.results]]
value = "'1.3.0'"

*/

RETURN string::semver::inc::minor("1.2.3");

-- '1.3.0'
