/**[test]

[[test.results]]
value = "NONE"

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
f"my_bucket:/other_book.txt".put("Or were there? I don't quite remember.");
-- Rename to my_book.txt, overwriting existing file of the same name
f"my_bucket:/other_book.txt".rename("my_book.txt");
<string>f"my_bucket:/my_book.txt".get();
