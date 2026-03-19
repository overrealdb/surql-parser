/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "100"

*/

DEFINE SEQUENCE mySeq2 BATCH 1000 START 100 TIMEOUT 5s;
sequence::nextval('mySeq2');

-- 100
