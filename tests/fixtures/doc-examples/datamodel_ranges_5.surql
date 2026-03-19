CREATE |person:20000| SET age = (rand::float() * 120).round() RETURN NONE;

-- Assign output to a parameter so the SELECT output is not displayed
LET $_ = SELECT * FROM person WHERE age > 18 AND age < 65;
LET $_ = SELECT * FROM person WHERE age in 18..=65;
