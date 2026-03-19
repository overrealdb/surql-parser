SELECT * FROM user VERSION time::now();

LET $now = time::now();
SELECT * FROM user VERSION $now;

DEFINE FUNCTION fn::yesterday() { time::now() - 1d };
SELECT * FROM user VERSION fn::yesterday();
