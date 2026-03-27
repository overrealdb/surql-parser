/**[test]

[[test.results]]
value = "'9.2.3'"

*/

RETURN string::semver::set::major("1.2.3", 9);

-- '9.2.3'
