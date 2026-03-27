-- Assuming the CREATE statements from the previous example
LET $founders = (SELECT * FROM person);
RETURN $founders.name;
