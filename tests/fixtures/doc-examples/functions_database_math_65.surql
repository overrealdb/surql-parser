/**[test]

[[test.results]]
value = "20.7dec"

[[test.results]]
value = "20.7dec"

*/

// Classic array filtering
math::sum([0,NONE,10dec,10.7,NULL][? $this]);
// array::map() function
math::sum([0, NONE, 10dec, 10.7, NULL].map(|$num| $num ?? 0));

-- 20.7dec
