/**[test]

[[test.results]]
value = "d'+262142-12-31T23:59:59.999999999Z'"

[[test.results]]
error = ""Failed to compute: \"1ns + d'+262142-12-31T23:59:59.999999999Z'\", as the operation results in an arithmetic overflow.""

[[test.results]]
value = "true"

*/

time::maximum;

time::maximum + 1ns;

time::now() IN time::minimum..time::maximum;
