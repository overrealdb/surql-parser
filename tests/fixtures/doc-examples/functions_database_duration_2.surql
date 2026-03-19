/**[test]

[[test.results]]
value = "584942417355y3w5d7h15s999ms999µs999ns"

[[test.results]]
error = "'Failed to compute: "584942417355y3w5d7h15s999ms999µs999ns + 1ns", as the operation results in an arithmetic overflow.'"

[[test.results]]
value = "true"

*/

duration::max;

duration::max + 1ns;

100y IN 0ns..duration::max
