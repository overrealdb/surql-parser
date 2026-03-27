-- Only used inside a DEFINE FIELD statement
DEFINE FIELD accessed_date ON person COMPUTED time::now();
