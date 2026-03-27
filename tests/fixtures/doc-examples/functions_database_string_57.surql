/**[test]

[[test.results]]
value = "'1.2.4'"

*/

RETURN string::semver::inc::patch("1.2.3");

-- '1.2.4'
