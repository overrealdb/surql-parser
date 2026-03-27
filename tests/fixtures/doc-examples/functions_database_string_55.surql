/**[test]

[[test.results]]
value = "'2.0.0'"

*/

RETURN string::semver::inc::major("1.2.3");

-- '2.0.0'
