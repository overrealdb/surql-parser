CREATE user:john SET name = 'John' VERSION time::now();

LET $now = time::now();
CREATE user:john_the_second SET name = 'John' VERSION $now;

DEFINE FUNCTION fn::yesterday() { time::now() - 1d };
CREATE user:john_the_third SET name = 'John' VERSION fn::yesterday();
