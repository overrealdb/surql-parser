/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "b"536F6D65206461746120696E73696465""

*/

DEFINE BUCKET my_bucket BACKEND "memory";

type::file("my_bucket", "file_name").put("Some data inside");
type::file("my_bucket", "file_name").get();
