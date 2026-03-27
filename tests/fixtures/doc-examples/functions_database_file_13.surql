/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = ""Or were there? I don't quite remember.""

*/

DEFINE BUCKET my_bucket BACKEND "memory";

f"my_bucket:/my_book.txt".put("Once there were four children whose names were Peter, Susan...");
f"my_bucket:/my_book.txt".put("Or were there? I don't quite remember.");
<string>f"my_bucket:/my_book.txt".get();
