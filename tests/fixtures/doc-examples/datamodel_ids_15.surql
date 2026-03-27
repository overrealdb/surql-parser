FOR $num IN 0..=100000 {
  CREATE person SET id = $num, num = $num  
};

-- Assign the output to an unused parameter
-- to avoid excessive output
LET $_ = SELECT * FROM person WHERE num IN 0..=1000;
LET $_ = SELECT * FROM person:0..=1000;
