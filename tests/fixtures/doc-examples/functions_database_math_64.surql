/**[test]

[[test.results]]
value = "[10dec, 10.7f]"

[[test.results]]
value = "[0, 0, 10dec, 10.7f, 0]"

*/

// Classic array filtering, removes NONE / NULL
[0,NONE,10dec,10.7,NULL][? $this];
// array::map() function, turns NONE / NULL to 0
[0, NONE, 10dec, 10.7, NULL].map(|$num| $num ?? 0);
