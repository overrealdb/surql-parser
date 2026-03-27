/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

*/

-- Specify the namespace and database for the user
USE NS abcum DB app_vitalsense;
-- Create the user with a viewer role and some example durations
DEFINE USER username ON DATABASE PASSWORD '123456' ROLES VIEWER DURATION FOR SESSION 5d, FOR TOKEN 2h;
