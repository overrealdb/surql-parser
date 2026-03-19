-- Just a RELATION table, no constraints on the type of table
DEFINE TABLE likes TYPE RELATION;

-- Define a relation table, and constrain the type of relation which can be stored
DEFINE TABLE likes TYPE RELATION FROM user TO post;
-- OR use IN and OUT alternatively to FROM and TO
DEFINE TABLE likes TYPE RELATION IN user OUT post;
-- To allow a link to one of a possible set of record types, use the | operator
DEFINE TABLE likes TYPE RELATION FROM user TO post|video;
DEFINE TABLE likes TYPE RELATION IN user OUT post|video;
