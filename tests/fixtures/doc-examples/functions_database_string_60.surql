/**[test]

[[test.results]]
value = "'1.2.9'"

*/

RETURN string::semver::set::patch("1.2.3", 9);

-- '1.2.9'
