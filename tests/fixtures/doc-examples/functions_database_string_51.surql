/**[test]

[[test.results]]
value = "-1"

*/

RETURN string::semver::compare("1.0.0", "1.3.5");
-- Returns -1
RETURN string::semver::compare("1.0.0", "1.0.0");
-- Returns 0
RETURN string::semver::compare("3.0.0-beta.4", "2.6.0");
-- Returns 1
