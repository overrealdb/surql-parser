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
value = "[{ file: f"my_bucket:/awesome_book", size: 19, updated: d'2025-10-07T01:13:20.226926Z' }, { file: f"my_bucket:/some_book", size: 39, updated: d'2025-10-07T01:13:20.227320Z' }]
"
skip-datetime = true

*/

DEFINE BUCKET my_bucket BACKEND "memory";

f"my_bucket:/some_book".put("Once upon a time...");
f"my_bucket:/some_book".rename("awesome_book");
f"my_bucket:/some_book".put("In a hole in the ground lived a Hobbit.");
file::list("my_bucket");
