/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "'Once there were four children whose names were Peter, Susan...'"

*/

DEFINE BUCKET my_bucket BACKEND "memory";

-- Creates file and adds data
f"my_bucket:/my_book.txt".put_if_not_exists("Once there were four children whose names were Peter, Susan...");
-- Does nothing
f"my_bucket:/my_book.txt".put_if_not_exists("Or were there? I don't quite remember.");
<string>f"my_bucket:/my_book.txt".get();
