/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "b"536F6D65207465787420696E73696465""

[[test.results]]
value = "'Some text inside'"

*/

DEFINE BUCKET my_bucket BACKEND "memory";
f"my_bucket:/some_file.txt".put("Some text inside");
f"my_bucket:/some_file.txt".get();
<string>f"my_bucket:/some_file.txt".get();
