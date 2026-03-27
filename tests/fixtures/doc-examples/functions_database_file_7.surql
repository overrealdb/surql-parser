/**[test]

[[test.results]]
value = "'/file_name'"

[[test.results]]
value = "NONE"

[[test.results]]
value = "'/file_name'"

*/
file::key(f"my_bucket:/file_name");

DEFINE PARAM $SOME_DATABASE_FILE VALUE f"my_bucket:/file_name";
file::key($SOME_DATABASE_FILE);
