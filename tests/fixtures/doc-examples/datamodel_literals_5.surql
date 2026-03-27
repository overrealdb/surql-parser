/**[test]

[[test.results]]
value = "NONE"

*/

DEFINE FIELD error_info ON TABLE information TYPE
	{ Continue:    { message: "" }} |
	{ Retry: { error: "Retrying", after: duration }} |
	{ Deprecated:  { message: string }};
