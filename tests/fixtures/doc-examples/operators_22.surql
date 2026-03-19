/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "true"

*/

let $threshold = 10;

string::similarity::smithwaterman("test text", "Test") > $threshold;
-- true
