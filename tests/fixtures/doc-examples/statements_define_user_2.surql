/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

*/

-- Specify the namespace
USE NS abcum;
-- Create the user with an editor role and some example durations
DEFINE USER username ON NAMESPACE PASSWORD '123456' ROLES EDITOR DURATION FOR SESSION 12h, FOR TOKEN 1m;
