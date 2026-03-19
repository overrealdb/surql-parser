DEFINE SEQUENCE mySeq2 BATCH 1000 START 100 TIMEOUT 5s;
sequence::nextval('mySeq2');
-- Output: 100

DEFINE SEQUENCE mySeq3 BATCH 1000 START 100 TIMEOUT 0ns;
sequence::nextval('mySeq3');
-- Possible output: 'The query was not executed because it exceeded the timeout'
