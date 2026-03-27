DEFINE SEQUENCE mySeq3 BATCH 1000 START 100 TIMEOUT 0ns;
INFO FOR DB.sequences;
sequence::nextval('mySeq3');

ALTER SEQUENCE mySeq3 TIMEOUT 100ms;
sequence::nextval('mySeq3');
