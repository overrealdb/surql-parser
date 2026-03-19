/**[test]

[[test.results]]
value = "'my_bucket'"

[[test.results]]
value = "NONE"

[[test.results]]
value = "'my_bucket'"

*/
file::bucket(f"my_bucket:/file_name");

DEFINE PARAM $SOME_DATABASE_FILE VALUE f"my_bucket:/file_name";
file::bucket($SOME_DATABASE_FILE);
