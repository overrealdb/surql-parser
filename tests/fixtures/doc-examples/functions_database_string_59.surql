/**[test]

[[test.results]]
value = "'1.9.3'"

*/

RETURN string::semver::set::minor("1.2.3", 9);

-- '1.9.3'
